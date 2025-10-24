use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_callback, wl_compositor, wl_output, wl_registry, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::backend::wayland::output::Output;
use crate::backend::wayland::surface::WaylandSurface;
use crate::config::watcher::ConfigWatcher;
use crate::config::Config;
use crate::core::power::PowerManager;
use crate::core::types::VideoSource;
use crate::ctl::ipc_server::IpcServer;
use crate::ctl::protocol::IpcCommand;
use crate::video::egl::EglContext;
use crate::video::frame_timing::FrameTiming;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

pub struct AppState {
    pub config: Config,
    pub config_path: Option<PathBuf>,
    pub config_watcher: Option<ConfigWatcher>,
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub xdg_output_manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
    pub outputs: HashMap<u32, Output>,
    pub surfaces: HashMap<u32, WaylandSurface>,
    pub running: bool,
    pub egl_context: Option<EglContext>,
    pub power_manager: PowerManager,
    pub last_frame_time: std::time::Instant,
    pub frame_timing: FrameTiming,
    pub ipc_server: Option<IpcServer>,
    pub command_rx: Option<Receiver<IpcCommand>>,
}

impl AppState {
    fn new(config: Config) -> Self {
        let target_fps = config.power.max_fps;
        Self {
            config,
            config_path: None,
            config_watcher: None,
            compositor: None,
            layer_shell: None,
            xdg_output_manager: None,
            outputs: HashMap::new(),
            surfaces: HashMap::new(),
            running: true,
            egl_context: None,
            power_manager: PowerManager::new(),
            last_frame_time: std::time::Instant::now(),
            frame_timing: FrameTiming::new(target_fps),
            ipc_server: None,
            command_rx: None,
        }
    }

    fn create_surface_for_output(&mut self, output_id: u32, qh: &QueueHandle<Self>) -> Result<()> {
        if self.surfaces.contains_key(&output_id) {
            return Ok(());
        }

        let output = match self.outputs.get(&output_id) {
            Some(o) => o,
            None => {
                warn!("Output {} not found", output_id);
                return Ok(());
            }
        };

        let compositor = self
            .compositor
            .as_ref()
            .context("Compositor not available")?;
        let layer_shell = self
            .layer_shell
            .as_ref()
            .context("Layer shell not available")?;

        let wl_surface = compositor.create_surface(qh, ());
        let effective_config = self.config.for_output(&output.info.name);

        let surface = WaylandSurface::new(
            wl_surface,
            layer_shell,
            output_id,
            output.info.clone(),
            effective_config,
            &output.wl_output,
            qh,
        )?;

        info!("Created surface for output: {}", output.info.name);
        self.surfaces.insert(output_id, surface);

        Ok(())
    }

    /// Apply power management: check battery, pause/resume players
    fn apply_power_management(&mut self) {
        let power_config = &self.config.power;

        // Check if pause_on_battery is enabled and we're on battery
        if power_config.pause_on_battery && self.power_manager.is_on_battery() {
            // Pause all players
            #[cfg(feature = "video-mpv")]
            for surface in self.surfaces.values_mut() {
                if let Err(e) = surface.pause_playback() {
                    warn!("Failed to pause playback: {}", e);
                }
            }
            debug!("Paused playback due to battery power");
        } else {
            // Resume all players
            #[cfg(feature = "video-mpv")]
            for surface in self.surfaces.values_mut() {
                if let Err(e) = surface.resume_playback() {
                    warn!("Failed to resume playback: {}", e);
                }
            }
        }
    }

    /// Check if FPS limiting should throttle rendering
    fn should_throttle_fps(&mut self) -> bool {
        let max_fps = self.config.power.max_fps;

        if max_fps == 0 {
            return false; // No FPS limit
        }

        let frame_duration = std::time::Duration::from_secs_f64(1.0 / max_fps as f64);
        let elapsed = self.last_frame_time.elapsed();

        if elapsed < frame_duration {
            true // Throttle
        } else {
            self.last_frame_time = std::time::Instant::now();
            false // Allow render
        }
    }

    /// Handle IPC command
    fn handle_command(&mut self, command: IpcCommand) {
        use crate::ctl::protocol::IpcCommand;

        match command {
            IpcCommand::Pause { output } => {
                self.handle_pause_command(output);
            }
            IpcCommand::Resume { output } => {
                self.handle_resume_command(output);
            }
            IpcCommand::Seek { output, time } => {
                self.handle_seek_command(output, time);
            }
            IpcCommand::SwitchSource { output, source } => {
                self.handle_switch_source_command(output, source);
            }
            IpcCommand::SetPlaybackRate { output, rate } => {
                self.handle_set_rate_command(output, rate);
            }
            IpcCommand::SetVolume { output, volume } => {
                self.handle_set_volume_command(output, volume);
            }
            IpcCommand::ToggleMute { output } => {
                self.handle_toggle_mute_command(output);
            }
            IpcCommand::SetLayout { output, layout } => {
                self.handle_set_layout_command(output, layout);
            }
            IpcCommand::GetStatus => {
                self.handle_get_status_command();
            }
            IpcCommand::ReloadConfig => {
                self.handle_reload_config_command();
            }
            IpcCommand::Quit => {
                info!("Received quit command");
                self.running = false;
            }
        }
    }

    /// Handle pause command
    fn handle_pause_command(&mut self, output: Option<String>) {
        #[cfg(feature = "video-mpv")]
        {
            if let Some(output_name) = output {
                // Pause specific output
                for surface in self.surfaces.values_mut() {
                    if surface.output_info.name == output_name {
                        if let Err(e) = surface.pause_playback() {
                            warn!("Failed to pause {}: {}", output_name, e);
                        } else {
                            info!("Paused playback on {}", output_name);
                        }
                        return;
                    }
                }
                warn!("Output not found: {}", output_name);
            } else {
                // Pause all outputs
                for surface in self.surfaces.values_mut() {
                    if let Err(e) = surface.pause_playback() {
                        warn!("Failed to pause {}: {}", surface.output_info.name, e);
                    }
                }
                info!("Paused all outputs");
            }
        }
    }

    /// Handle resume command
    fn handle_resume_command(&mut self, output: Option<String>) {
        #[cfg(feature = "video-mpv")]
        {
            if let Some(output_name) = output {
                // Resume specific output
                for surface in self.surfaces.values_mut() {
                    if surface.output_info.name == output_name {
                        if let Err(e) = surface.resume_playback() {
                            warn!("Failed to resume {}: {}", output_name, e);
                        } else {
                            info!("Resumed playback on {}", output_name);
                        }
                        return;
                    }
                }
                warn!("Output not found: {}", output_name);
            } else {
                // Resume all outputs
                for surface in self.surfaces.values_mut() {
                    if let Err(e) = surface.resume_playback() {
                        warn!("Failed to resume {}: {}", surface.output_info.name, e);
                    }
                }
                info!("Resumed all outputs");
            }
        }
    }

    /// Handle seek command
    fn handle_seek_command(&mut self, output: String, time: f64) {
        #[cfg(feature = "video-mpv")]
        {
            for surface in self.surfaces.values_mut() {
                if surface.output_info.name == output {
                    if let Err(e) = surface.seek(time) {
                        warn!("Failed to seek {}: {}", output, e);
                    } else {
                        info!("Seeked {} to {:.2}s", output, time);
                    }
                    return;
                }
            }
            warn!("Output not found: {}", output);
        }
    }

    /// Handle switch source command
    fn handle_switch_source_command(&mut self, output: String, source: VideoSource) {
        #[cfg(feature = "video-mpv")]
        {
            for surface in self.surfaces.values_mut() {
                if surface.output_info.name == output {
                    // Extract source path for switch_source (which expects string)
                    let source_path = match &source {
                        VideoSource::File { path } => path.clone(),
                        VideoSource::Url { url } => url.clone(),
                        VideoSource::Rtsp { url } => url.clone(),
                        VideoSource::Directory { path } => path.clone(),
                        VideoSource::Pipe { path } => {
                            if path.is_empty() {
                                "fd://0".to_string()
                            } else {
                                path.clone()
                            }
                        }
                        VideoSource::ImageSequence { path, .. } => path.clone(),
                        VideoSource::WeProject { path } => path.clone(),
                    };

                    if let Err(e) = surface.switch_source(&source_path) {
                        warn!("Failed to switch source on {}: {}", output, e);
                    } else {
                        info!("Switched {} to source: {:?}", output, source);
                    }
                    return;
                }
            }
            warn!("Output not found: {}", output);
        }
    }

    /// Handle set playback rate command
    fn handle_set_rate_command(&mut self, output: String, rate: f64) {
        #[cfg(feature = "video-mpv")]
        {
            for surface in self.surfaces.values_mut() {
                if surface.output_info.name == output {
                    if let Err(e) = surface.set_playback_rate(rate) {
                        warn!("Failed to set rate on {}: {}", output, e);
                    } else {
                        info!("Set playback rate of {} to {:.2}x", output, rate);
                    }
                    return;
                }
            }
            warn!("Output not found: {}", output);
        }
    }

    /// Handle set volume command
    fn handle_set_volume_command(&mut self, output: String, volume: f64) {
        #[cfg(feature = "video-mpv")]
        {
            for surface in self.surfaces.values_mut() {
                if surface.output_info.name == output {
                    if let Err(e) = surface.set_volume(volume) {
                        warn!("Failed to set volume on {}: {}", output, e);
                    } else {
                        info!("Set volume of {} to {:.2}", output, volume);
                    }
                    return;
                }
            }
            warn!("Output not found: {}", output);
        }
    }

    /// Handle toggle mute command
    fn handle_toggle_mute_command(&mut self, output: String) {
        #[cfg(feature = "video-mpv")]
        {
            for surface in self.surfaces.values_mut() {
                if surface.output_info.name == output {
                    if let Err(e) = surface.toggle_mute() {
                        warn!("Failed to toggle mute on {}: {}", output, e);
                    } else {
                        info!("Toggled mute on {}", output);
                    }
                    return;
                }
            }
            warn!("Output not found: {}", output);
        }
    }

    /// Handle set layout command
    fn handle_set_layout_command(&mut self, output: String, layout: String) {
        use crate::core::types::LayoutMode;

        let layout_mode = match layout.to_lowercase().as_str() {
            "centre" | "center" => LayoutMode::Centre,
            "stretch" => LayoutMode::Stretch,
            "contain" | "fit" => LayoutMode::Contain,
            "fill" => LayoutMode::Fill,
            "cover" => LayoutMode::Cover,
            _ => {
                warn!(
                    "Unknown layout mode: {}. Valid: centre, stretch, contain, fill, cover",
                    layout
                );
                return;
            }
        };

        for surface in self.surfaces.values_mut() {
            if surface.output_info.name == output {
                surface.set_layout(layout_mode);
                info!("Set layout of {} to {:?}", output, layout_mode);
                return;
            }
        }
        warn!("Output not found: {}", output);
    }

    /// Handle get status command (prints to log for now)
    fn handle_get_status_command(&mut self) {
        #[cfg(feature = "video-mpv")]
        {
            info!("=== Wayvid Status ===");
            info!("Running: {}", self.running);
            info!("Outputs: {}", self.surfaces.len());
            for (id, surface) in &self.surfaces {
                let status = surface.get_status();
                info!(
                    "  [{}] {} ({}x{}) - Layout: {:?}, Playing: {:?}",
                    id,
                    surface.output_info.name,
                    surface.output_info.width,
                    surface.output_info.height,
                    surface.config.layout,
                    status,
                );
            }
            info!("====================");
        }
    }

    /// Handle reload config command
    fn handle_reload_config_command(&mut self) {
        if let Err(e) = self.reload_config() {
            warn!("Failed to reload config: {}", e);
        } else {
            info!("Config reloaded successfully via IPC command");
        }
    }

    /// Reload configuration from file
    fn reload_config(&mut self) -> Result<()> {
        let config_path = self
            .config_path
            .as_ref()
            .context("No config path available for reload")?;

        info!("Reloading config from: {}", config_path.display());

        // Load new config
        let new_config = Config::from_file(config_path)?;

        // Apply to all surfaces
        for (output_id, surface) in self.surfaces.iter_mut() {
            let output_info = &surface.output_info;

            // Get effective config for this output
            let effective_config = new_config.for_output(&output_info.name);

            info!(
                "  Applying config to output: {} ({})",
                output_info.name, output_id
            );

            // Apply new config to surface
            #[cfg(feature = "video-mpv")]
            {
                // Update layout
                surface.set_layout(effective_config.layout);

                // Update playback rate
                if let Err(e) = surface.set_playback_rate(effective_config.playback_rate) {
                    warn!("    Failed to set playback rate: {}", e);
                }

                // Update volume
                if let Err(e) = surface.set_volume(effective_config.volume) {
                    warn!("    Failed to set volume: {}", e);
                }

                // Switch source if changed
                if surface.config.source != effective_config.source {
                    let new_source = effective_config.source.get_mpv_path();
                    info!("    Switching source to: {}", new_source);
                    if let Err(e) = surface.switch_source(&new_source) {
                        warn!("    Failed to switch source: {}", e);
                    }
                }
            }

            // Update stored config
            surface.config = effective_config;
        }

        // Update stored config
        self.config = new_config;

        Ok(())
    }
} // Dispatch implementations
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for AppState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                info!("Registry global: {} v{} (id={})", interface, version, name);

                match &interface[..] {
                    "wl_compositor" => {
                        let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(
                            name,
                            version.min(4),
                            qh,
                            (),
                        );
                        state.compositor = Some(compositor);
                        info!("Bound wl_compositor");
                    }
                    "zwlr_layer_shell_v1" => {
                        let layer_shell = registry
                            .bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                                name,
                                version.min(4),
                                qh,
                                (),
                            );
                        state.layer_shell = Some(layer_shell);
                        info!("Bound zwlr_layer_shell_v1");
                    }
                    "wl_output" => {
                        let wl_output = registry.bind::<wl_output::WlOutput, _, _>(
                            name,
                            version.min(3),
                            qh,
                            name,
                        );

                        let output = Output::new(wl_output, format!("output-{}", name));
                        state.outputs.insert(name, output);
                        info!("Added output: {}", name);

                        // Get xdg_output if manager is available
                        if let Some(ref manager) = state.xdg_output_manager {
                            if let Some(output) = state.outputs.get_mut(&name) {
                                let xdg_output =
                                    manager.get_xdg_output(&output.wl_output, qh, name);
                                output.set_xdg_output(xdg_output);
                                debug!("Requested xdg_output for output {}", name);
                            }
                        }
                    }
                    "zxdg_output_manager_v1" => {
                        let manager = registry
                            .bind::<zxdg_output_manager_v1::ZxdgOutputManagerV1, _, _>(
                                name,
                                version.min(3),
                                qh,
                                (),
                            );
                        state.xdg_output_manager = Some(manager);
                        info!("Bound zxdg_output_manager_v1");
                    }
                    _ => {}
                }
            }
            wl_registry::Event::GlobalRemove { name } => {
                info!("Registry global removed: {}", name);

                // Remove output and associated surface
                if state.outputs.remove(&name).is_some() {
                    info!("Removed output: {}", name);

                    // Destroy surface associated with this output
                    if let Some(_surface) = state.surfaces.remove(&name) {
                        info!("Destroyed surface for output: {}", name);
                    }
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for AppState {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zwlr_layer_surface_v1::Event;

        match event {
            Event::Configure {
                serial,
                width,
                height,
            } => {
                debug!(
                    "Layer surface configure: {}x{} (serial: {})",
                    width, height, serial
                );

                // Find the surface and configure it
                let egl_ctx = state.egl_context.as_ref();
                for surface in state.surfaces.values_mut() {
                    if surface.layer_surface == *layer_surface {
                        surface.configure(width, height, serial, egl_ctx);
                        break;
                    }
                }
            }
            Event::Closed => {
                info!("Layer surface closed");
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for AppState {
    fn event(
        state: &mut Self,
        _wl_output: &wl_output::WlOutput,
        event: wl_output::Event,
        output_id: &u32,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wl_output::Event;

        match event {
            Event::Geometry { x, y, .. } => {
                if let Some(output) = state.outputs.get_mut(output_id) {
                    output.update_position(x, y);
                    debug!("Output {} geometry: position=({}, {})", output_id, x, y);
                }
            }
            Event::Mode { width, height, .. } => {
                if let Some(output) = state.outputs.get_mut(output_id) {
                    output.update_mode(width, height);
                    info!("Output {} mode: {}x{}", output_id, width, height);
                }
            }
            Event::Scale { factor } => {
                if let Some(output) = state.outputs.get_mut(output_id) {
                    output.update_scale(factor);
                    info!("Output {} scale: {}", output_id, factor);
                }
            }
            Event::Done => {
                debug!("Output {} done", output_id);
                // DON'T create surface yet - wait for XDG name first
                // Surface creation will happen after XDG roundtrip in run()
            }
            _ => {}
        }
    }
}

// Frame callback handler for vsync
impl Dispatch<wl_callback::WlCallback, u32> for AppState {
    fn event(
        state: &mut Self,
        _callback: &wl_callback::WlCallback,
        event: wl_callback::Event,
        output_id: &u32,
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wl_callback::Event;

        if let Event::Done { callback_data: _ } = event {
            // Frame callback triggered - mark that frame is ready to render
            if let Some(surface) = state.surfaces.get_mut(output_id) {
                surface.on_frame_ready();
            }
        }
    }
}

pub fn run(config: Config, config_path: Option<PathBuf>) -> Result<()> {
    // Start startup time measurement
    let startup_start = std::time::Instant::now();

    info!("Starting wayvid Wayland backend");

    let conn = Connection::connect_to_env().context("Failed to connect to Wayland compositor")?;

    let (globals, mut event_queue) =
        registry_queue_init::<AppState>(&conn).context("Failed to initialize registry")?;

    let qh = event_queue.handle();
    let mut state = AppState::new(config);

    // Store config path for hot reload
    state.config_path = config_path.clone();

    // Start config file watcher if path is provided
    if let Some(ref path) = config_path {
        match ConfigWatcher::watch(path.clone()) {
            Ok(watcher) => {
                info!("  ✓ Config watcher started for: {}", path.display());
                state.config_watcher = Some(watcher);
            }
            Err(e) => {
                warn!("  ✗ Failed to start config watcher: {}", e);
                warn!("    Continuing without hot reload support");
            }
        }
    }

    // Start IPC server
    info!("Starting IPC server...");
    match IpcServer::start() {
        Ok((server, rx)) => {
            info!("  ✓ IPC server listening on: {:?}", server.socket_path());
            state.ipc_server = Some(server);
            state.command_rx = Some(rx);
        }
        Err(e) => {
            warn!("  ✗ Failed to start IPC server: {}", e);
            warn!("    Continuing without IPC support");
        }
    }

    // Bind necessary globals
    info!("Binding Wayland globals...");

    // Bind compositor
    let compositor: wl_compositor::WlCompositor = globals
        .bind(&qh, 1..=4, ())
        .context("Failed to bind wl_compositor")?;
    state.compositor = Some(compositor);
    info!("  ✓ wl_compositor");

    // Bind layer shell
    let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 = globals
        .bind(&qh, 1..=4, ())
        .context("Failed to bind zwlr_layer_shell_v1")?;
    state.layer_shell = Some(layer_shell);
    info!("  ✓ zwlr_layer_shell_v1");

    // Bind xdg_output_manager if available (must bind BEFORE outputs)
    for global in globals.contents().with_list(|list| list.to_vec()) {
        if global.interface == "zxdg_output_manager_v1" {
            let manager: zxdg_output_manager_v1::ZxdgOutputManagerV1 = globals
                .registry()
                .bind(global.name, global.version.min(3), &qh, ());
            state.xdg_output_manager = Some(manager);
            info!("  ✓ zxdg_output_manager_v1");
            break;
        }
    }

    // Bind outputs - iterate through globals list
    let mut output_count = 0;
    for global in globals.contents().with_list(|list| list.to_vec()) {
        if global.interface == "wl_output" {
            let wl_output: wl_output::WlOutput =
                globals
                    .registry()
                    .bind(global.name, global.version.min(3), &qh, global.name);
            let output = Output::new(wl_output, format!("output-{}", global.name));
            state.outputs.insert(global.name, output);
            output_count += 1;
        }
    }
    info!("  ✓ {} outputs", output_count);

    // Initialize EGL context
    info!("Initializing EGL context...");
    let wl_display_ptr = conn.backend().display_ptr() as *mut std::ffi::c_void;
    match EglContext::new(wl_display_ptr) {
        Ok(egl_ctx) => {
            state.egl_context = Some(egl_ctx);
            info!("  ✓ EGL context initialized");
        }
        Err(e) => {
            warn!("  ✗ Failed to initialize EGL: {}", e);
            warn!("    Continuing without OpenGL rendering");
        }
    }

    // Initial roundtrip to get output information
    info!("Performing initial roundtrip to get output info...");
    event_queue
        .roundtrip(&mut state)
        .context("Initial roundtrip failed")?;

    info!("First roundtrip complete");
    info!(
        "  Compositor: {}",
        if state.compositor.is_some() {
            "✓"
        } else {
            "✗"
        }
    );
    info!(
        "  Layer shell: {}",
        if state.layer_shell.is_some() {
            "✓"
        } else {
            "✗"
        }
    );
    info!("  Outputs discovered: {}", state.outputs.len());
    info!(
        "  XDG output manager: {}",
        if state.xdg_output_manager.is_some() {
            "✓"
        } else {
            "✗"
        }
    );

    if state.compositor.is_none() {
        anyhow::bail!("wl_compositor not available");
    }
    if state.layer_shell.is_none() {
        anyhow::bail!("zwlr_layer_shell_v1 not available - compositor not supported");
    }

    // Request xdg_output for all outputs if manager is available
    if let Some(ref manager) = state.xdg_output_manager {
        info!("Requesting XDG output info for all outputs...");
        let output_ids: Vec<u32> = state.outputs.keys().copied().collect();
        for output_id in output_ids {
            if let Some(output) = state.outputs.get_mut(&output_id) {
                let xdg_output = manager.get_xdg_output(&output.wl_output, &qh, output_id);
                output.set_xdg_output(xdg_output);
                debug!("Requested xdg_output for output {}", output_id);
            }
        }
        
        // Additional roundtrip to receive XDG output events
        debug!("Performing roundtrip to receive XDG output names...");
        event_queue
            .roundtrip(&mut state)
            .context("XDG output roundtrip failed")?;
        debug!("XDG output roundtrip complete");
    } else {
        warn!("  XDG output manager not available - using generic output names");
    }

    // Now create surfaces with correct output names (after XDG names are received)
    info!("Creating surfaces for all outputs...");
    let output_ids: Vec<u32> = state.outputs.keys().copied().collect();
    for output_id in output_ids {
        if let Err(e) = state.create_surface_for_output(output_id, &qh) {
            warn!("Failed to create surface for output {}: {}", output_id, e);
        }
    }

    // Another roundtrip to create surfaces
    event_queue
        .roundtrip(&mut state)
        .context("Second roundtrip failed")?;

    info!("Created {} surfaces", state.surfaces.len());

    // Request initial frame callbacks for all surfaces
    info!("Requesting initial frame callbacks...");
    let qh = event_queue.handle();
    for surface in state.surfaces.values_mut() {
        surface.request_frame(&qh);
        // Mark frame pending so first render happens
        surface.on_frame_ready();
    }

    // Initial render (triggers lazy initialization)
    info!("Performing initial render (triggers lazy initialization)...");
    let egl_ctx = state.egl_context.as_ref();
    for surface in state.surfaces.values_mut() {
        if let Err(e) = surface.render(egl_ctx) {
            warn!("Initial render error: {}", e);
        }
        // Request next frame after initial render
        surface.request_frame(&qh);
    }

    // Measure and report startup time
    let startup_duration = startup_start.elapsed();
    info!(
        "✅ Startup complete in {:.1}ms",
        startup_duration.as_millis()
    );
    info!("   Lazy initialization: resources allocated on first render");

    // Frame statistics reporting
    let mut last_stats_report = std::time::Instant::now();
    const STATS_REPORT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

    // Main event loop with vsync
    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .context("Event dispatch failed")?;

        // Process IPC commands (non-blocking)
        let commands: Vec<IpcCommand> = if let Some(ref rx) = state.command_rx {
            let mut cmds = Vec::new();
            while let Ok(command) = rx.try_recv() {
                cmds.push(command);
            }
            cmds
        } else {
            Vec::new()
        };

        for command in commands {
            info!("Processing IPC command: {:?}", command);
            state.handle_command(command);
        }

        // Check for config file changes (non-blocking)
        if let Some(ref watcher) = state.config_watcher {
            if let Some(changed_path) = watcher.try_recv() {
                info!("Config file changed: {}", changed_path.display());
                if let Err(e) = state.reload_config() {
                    warn!("Failed to reload config: {}", e);
                } else {
                    info!("Config reloaded successfully");
                }
            }
        }

        // Apply power management (battery check, pause/resume)
        state.apply_power_management();

        // Check FPS throttling
        if state.should_throttle_fps() {
            continue; // Skip rendering this frame
        }

        // Begin frame timing measurement
        state.frame_timing.begin_frame();

        // Check if we should skip this frame due to overload
        if state.frame_timing.should_skip_frame() {
            state.frame_timing.record_skip();
            continue; // Skip this frame to reduce load
        }

        // Render surfaces that have pending frames (triggered by frame callbacks)
        let egl_ctx = state.egl_context.as_ref();
        let qh = event_queue.handle();
        for surface in state.surfaces.values_mut() {
            // Check if frame is ready before rendering
            let should_render = surface.has_frame_pending();

            if let Err(e) = surface.render(egl_ctx) {
                warn!("Render error: {}", e);
            }

            // Request next frame if we just rendered
            if should_render {
                surface.request_frame(&qh);
            }
        }

        // End frame timing measurement
        state.frame_timing.end_frame();

        // Periodically report frame statistics
        if last_stats_report.elapsed() >= STATS_REPORT_INTERVAL {
            let stats = state.frame_timing.get_stats();
            info!(
                "📊 Frame stats: {}/{} rendered/skipped ({:.1}% skip rate), load: {:.1}%, avg: {:.1}ms{}",
                stats.frames_rendered,
                stats.frames_skipped,
                stats.skip_percentage,
                stats.current_load_pct,
                stats.avg_frame_duration_ms,
                if stats.in_skip_mode { " [SKIP MODE]" } else { "" }
            );
            last_stats_report = std::time::Instant::now();
        }
    }

    // Final statistics
    let final_stats = state.frame_timing.get_stats();
    info!("📊 Final frame statistics:");
    info!("   Total frames: {}", final_stats.total_frames);
    info!("   Rendered: {}", final_stats.frames_rendered);
    info!("   Skipped: {}", final_stats.frames_skipped);
    info!("   Skip rate: {:.1}%", final_stats.skip_percentage);
    info!(
        "   Average frame time: {:.1}ms",
        final_stats.avg_frame_duration_ms
    );

    info!("Shutting down");
    Ok(())
}

// xdg_output_manager_v1 Dispatch
impl Dispatch<zxdg_output_manager_v1::ZxdgOutputManagerV1, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &zxdg_output_manager_v1::ZxdgOutputManagerV1,
        _: zxdg_output_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

// xdg_output_v1 Dispatch
impl Dispatch<zxdg_output_v1::ZxdgOutputV1, u32> for AppState {
    fn event(
        state: &mut Self,
        _: &zxdg_output_v1::ZxdgOutputV1,
        event: zxdg_output_v1::Event,
        output_id: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zxdg_output_v1::Event;

        match event {
            Event::Name { name } => {
                if let Some(output) = state.outputs.get_mut(output_id) {
                    info!("Output {} xdg_name: {}", output_id, name);
                    output.info.name = name;
                }
            }
            Event::Description { description } => {
                debug!("Output {} description: {}", output_id, description);
            }
            Event::LogicalPosition { x, y } => {
                if let Some(output) = state.outputs.get_mut(output_id) {
                    debug!("Output {} logical position: ({}, {})", output_id, x, y);
                    output.update_position(x, y);
                }
            }
            Event::LogicalSize { width, height } => {
                debug!("Output {} logical size: {}x{}", output_id, width, height);
                // Note: We use physical size from wl_output for rendering
            }
            Event::Done => {
                debug!("Output {} xdg_output done", output_id);
            }
            _ => {}
        }
    }
}
