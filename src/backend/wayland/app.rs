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

use crate::backend::niri::NiriClient;
use crate::backend::wayland::output::Output;
use crate::backend::wayland::surface::WaylandSurface;
use crate::config::watcher::ConfigWatcher;
use crate::config::Config;
use crate::core::power::PowerManager;
use crate::core::types::VideoSource;
use crate::ctl::ipc_server::{IpcRequest, IpcServer};
use crate::ctl::protocol::{IpcCommand, IpcResponse};
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
    #[allow(dead_code)]
    pub last_frame_time: std::time::Instant,
    pub frame_timing: FrameTiming,
    pub ipc_server: Option<IpcServer>,
    pub request_rx: Option<Receiver<IpcRequest>>,
    pub niri_client: Option<NiriClient>,
    pub focused_workspace: Option<u64>,
    /// Track if playback is currently paused due to power management
    playback_paused_for_battery: bool,
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
            request_rx: None,
            niri_client: None,
            focused_workspace: None,
            playback_paused_for_battery: false,
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
    /// Only takes action when state changes to avoid spam
    fn apply_power_management(&mut self) {
        let power_config = &self.config.power;

        // Check if pause_on_battery is enabled and we're on battery
        let should_pause = power_config.pause_on_battery && self.power_manager.is_on_battery();

        // Only take action if state changed
        if should_pause && !self.playback_paused_for_battery {
            // Transition to paused state
            self.playback_paused_for_battery = true;
            info!("Pausing playback due to battery power");
            #[cfg(feature = "video-mpv")]
            for surface in self.surfaces.values_mut() {
                if let Err(e) = surface.pause_playback() {
                    warn!("Failed to pause playback: {}", e);
                }
            }
        } else if !should_pause && self.playback_paused_for_battery {
            // Transition to playing state
            self.playback_paused_for_battery = false;
            info!("Resuming playback (AC power restored)");
            #[cfg(feature = "video-mpv")]
            for surface in self.surfaces.values_mut() {
                if let Err(e) = surface.resume_playback() {
                    warn!("Failed to resume playback: {}", e);
                }
            }
        }
        // If state hasn't changed, do nothing
    }

    /// Check if FPS limiting should throttle rendering
    /// Currently disabled - let Wayland vsync handle frame timing
    #[allow(dead_code)]
    fn should_throttle_fps(&mut self) -> bool {
        // Disabled: FPS throttle breaks the frame callback chain
        // TODO: Implement proper throttling that doesn't break callbacks
        false

        /*
        // Niri workspace optimization: reduce FPS when wallpaper workspace is not in focus
        // This only applies to Niri compositor
        if let Some(ref mut niri_client) = self.niri_client {
            if let Ok(Some(focused)) = niri_client.get_focused_workspace() {
                // Update focused workspace
                let was_focused = self.focused_workspace;
                self.focused_workspace = Some(focused);

                // Log workspace changes
                if was_focused != Some(focused) {
                    debug!("Focused workspace changed to {}", focused);
                }

                // For now, don't throttle based on workspace
                // TODO: Implement proper workspace-aware throttling when we have
                // a way to know which workspace the wallpaper is on
            }
        }

        // Normal FPS throttling based on max_fps config
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
        */
    }

    /// Handle IPC command (legacy method for internal use)
    #[allow(dead_code)]
    fn handle_command(&mut self, command: IpcCommand) {
        let _ = self.handle_command_with_response(command);
    }

    /// Handle IPC command and return response
    fn handle_command_with_response(&mut self, command: IpcCommand) -> IpcResponse {
        match command {
            IpcCommand::Pause { output } => {
                self.handle_pause_command(output);
                IpcResponse::Success { data: None }
            }
            IpcCommand::Resume { output } => {
                self.handle_resume_command(output);
                IpcResponse::Success { data: None }
            }
            IpcCommand::Seek { output, time } => {
                self.handle_seek_command(output, time);
                IpcResponse::Success { data: None }
            }
            IpcCommand::SwitchSource { output, source } => {
                self.handle_switch_source_command(output, source);
                IpcResponse::Success { data: None }
            }
            IpcCommand::SetSource { output, source } => {
                self.handle_set_source_command(output, source);
                IpcResponse::Success { data: None }
            }
            IpcCommand::SetPlaybackRate { output, rate } => {
                self.handle_set_rate_command(output, rate);
                IpcResponse::Success { data: None }
            }
            IpcCommand::SetVolume { output, volume } => {
                self.handle_set_volume_command(output, volume);
                IpcResponse::Success { data: None }
            }
            IpcCommand::ToggleMute { output } => {
                self.handle_toggle_mute_command(output);
                IpcResponse::Success { data: None }
            }
            IpcCommand::SetLayout { output, layout } => {
                self.handle_set_layout_command(output, layout);
                IpcResponse::Success { data: None }
            }
            IpcCommand::GetStatus => self.handle_get_status_response(),
            IpcCommand::ReloadConfig => {
                self.handle_reload_config_command();
                IpcResponse::Success { data: None }
            }
            IpcCommand::Quit => {
                info!("Received quit command");
                self.running = false;
                IpcResponse::Success { data: None }
            }
        }
    }

    /// Handle get status command and return response with data
    fn handle_get_status_response(&self) -> IpcResponse {
        use crate::ctl::protocol::{DaemonStatus, OutputStatus};

        let mut outputs = Vec::new();

        #[cfg(feature = "video-mpv")]
        {
            for surface in self.surfaces.values() {
                // get_status returns Option<(is_playing, current_time, duration)>
                let status = surface.get_status();
                let (playing, current_time, duration) = match status {
                    Some((is_playing, time, dur)) => (is_playing, time, dur),
                    None => (false, 0.0, 0.0),
                };

                outputs.push(OutputStatus {
                    name: surface.output_info.name.clone(),
                    width: surface.output_info.width,
                    height: surface.output_info.height,
                    playing,
                    paused: !playing,
                    current_time,
                    duration,
                    source: surface.config.source.get_mpv_path(),
                    layout: format!("{:?}", surface.config.layout),
                    volume: surface.config.volume,
                    muted: surface.config.mute,
                    playback_rate: surface.config.playback_rate,
                });
            }
        }

        let status = DaemonStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            outputs,
        };

        match serde_json::to_value(status) {
            Ok(data) => IpcResponse::Success { data: Some(data) },
            Err(e) => IpcResponse::Error {
                message: format!("Failed to serialize status: {}", e),
            },
        }
    }

    /// Handle IPC command (old method, deprecated)
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
            let egl_ctx = self.egl_context.as_ref();

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

                    if let Err(e) = surface.switch_source(&source_path, egl_ctx) {
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

    /// Handle set source command (simplified version) - also saves to config
    fn handle_set_source_command(&mut self, output: Option<String>, source_path: String) {
        use crate::core::types::VideoSource;

        // Parse source path to VideoSource
        let video_source =
            if source_path.starts_with("http://") || source_path.starts_with("https://") {
                VideoSource::Url {
                    url: source_path.clone(),
                }
            } else if source_path.starts_with("rtsp://") {
                VideoSource::Rtsp {
                    url: source_path.clone(),
                }
            } else {
                let path = shellexpand::tilde(&source_path).to_string();
                if std::path::Path::new(&path).is_dir() {
                    VideoSource::Directory { path }
                } else {
                    VideoSource::File { path }
                }
            };

        #[cfg(feature = "video-mpv")]
        {
            let targets: Vec<String> = if let Some(output_name) = output {
                vec![output_name]
            } else {
                self.surfaces
                    .keys()
                    .map(|&id| self.surfaces[&id].output_info.name.clone())
                    .collect()
            };

            let egl_ctx = self.egl_context.as_ref();

            for output_name in &targets {
                for surface in self.surfaces.values_mut() {
                    if &surface.output_info.name == output_name {
                        if let Err(e) = surface.switch_source(&source_path, egl_ctx) {
                            warn!("Failed to set source on {}: {}", output_name, e);
                        } else {
                            info!("Set {} to source: {}", output_name, source_path);
                        }
                        break;
                    }
                }
            }

            // Save to config for persistence
            for output_name in targets {
                self.config
                    .set_output_source(&output_name, video_source.clone());
            }

            // Save config to file
            if let Some(ref config_path) = self.config_path {
                if let Err(e) = self.config.save_to_file(config_path) {
                    warn!("Failed to save config: {}", e);
                } else {
                    info!("Configuration saved with new wallpaper setting");
                }
            }
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

        // Collect source changes that need to be applied
        // (we need to do this separately due to borrow checker)
        #[cfg(feature = "video-mpv")]
        let source_changes: Vec<(String, String)> = self
            .surfaces
            .values()
            .filter_map(|surface| {
                let effective_config = new_config.for_output(&surface.output_info.name);
                if surface.config.source != effective_config.source {
                    Some((
                        surface.output_info.name.clone(),
                        effective_config.source.get_mpv_path(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Apply to all surfaces (non-source changes)
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
            }

            // Update stored config
            surface.config = effective_config;
        }

        // Apply source changes separately
        #[cfg(feature = "video-mpv")]
        {
            let egl_ctx = self.egl_context.as_ref();
            for (output_name, new_source) in source_changes {
                info!(
                    "    Switching source for {} to: {}",
                    output_name, new_source
                );
                for surface in self.surfaces.values_mut() {
                    if surface.output_info.name == output_name {
                        if let Err(e) = surface.switch_source(&new_source, egl_ctx) {
                            warn!("    Failed to switch source: {}", e);
                        }
                        break;
                    }
                }
            }
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
        _qh: &QueueHandle<Self>,
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
// Note: In the simplified render loop, this callback is requested but not used
// to drive rendering. It's kept for potential future vsync-aware optimizations.
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
            // Frame callback triggered - compositor displayed our frame
            // In the simplified architecture, we don't use this to drive rendering,
            // but it could be used for vsync-aware optimizations in the future
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

    // Check for conflicting wallpaper managers
    crate::backend::wayland::conflicts::check_wallpaper_conflicts();

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
            state.request_rx = Some(rx);
        }
        Err(e) => {
            warn!("  ✗ Failed to start IPC server: {}", e);
            warn!("    Continuing without IPC support");
        }
    }

    // Connect to Niri if running
    if crate::backend::niri::is_niri() {
        info!("Detected Niri compositor");
        match NiriClient::connect() {
            Ok(mut client) => {
                // Get initial focused workspace
                match client.get_focused_workspace() {
                    Ok(workspace) => {
                        info!("  ✓ Niri integration enabled (workspace: {:?})", workspace);
                        state.focused_workspace = workspace;
                        state.niri_client = Some(client);
                    }
                    Err(e) => {
                        warn!("  ✗ Failed to get workspace info: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("  ✗ Failed to connect to Niri: {}", e);
                warn!("    Continuing without Niri integration");
            }
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
            let manager: zxdg_output_manager_v1::ZxdgOutputManagerV1 =
                globals
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

    // Initial render (triggers lazy initialization)
    // frame_pending is set to true in configure(), so first render will work
    info!("Performing initial render (triggers lazy initialization)...");
    let egl_ctx = state.egl_context.as_ref();
    let qh = event_queue.handle();
    for surface in state.surfaces.values_mut() {
        if let Err(e) = surface.render(egl_ctx, &qh) {
            warn!("Initial render error: {}", e);
        }
        // Note: render() now calls request_frame() internally before commit
    }

    // Measure and report startup time
    let startup_duration = startup_start.elapsed();
    info!(
        "✅ Startup complete in {:.1}ms",
        startup_duration.as_millis()
    );
    info!("   Lazy initialization: resources allocated on first render");

    // Notify systemd that daemon is ready
    #[cfg(target_os = "linux")]
    notify_systemd_ready();

    // Frame statistics reporting
    let mut last_stats_report = std::time::Instant::now();
    const STATS_REPORT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

    // Periodic check intervals (not every frame)
    let mut last_power_check = std::time::Instant::now();
    const POWER_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

    use std::os::fd::AsFd;
    use std::os::fd::AsRawFd;

    // Get the Wayland connection file descriptor for polling
    let wayland_fd = conn.as_fd().as_raw_fd();

    // Main event loop with polling
    while state.running {
        // Try to dispatch pending events first (non-blocking)
        if let Err(e) = event_queue.dispatch_pending(&mut state) {
            warn!("Failed to dispatch pending events: {}", e);
        }

        // Flush pending requests
        if let Err(e) = event_queue.flush() {
            warn!("Failed to flush event queue: {}", e);
        }

        // Prepare to read from Wayland connection
        if let Some(guard) = event_queue.prepare_read() {
            // Use libc poll with moderate timeout
            let mut poll_fd = libc::pollfd {
                fd: wayland_fd,
                events: libc::POLLIN,
                revents: 0,
            };

            // Use 10ms timeout - balances responsiveness and CPU usage
            // Frame callbacks from compositor arrive at ~30fps (every ~33ms)
            let ret = unsafe { libc::poll(&mut poll_fd, 1, 10) };

            if ret > 0 && (poll_fd.revents & libc::POLLIN) != 0 {
                // Events available, read them
                if let Err(e) = guard.read() {
                    warn!("Failed to read Wayland events: {}", e);
                }
            } else {
                // Timeout or error - drop guard without reading
                drop(guard);
            }
        }

        // Dispatch any new events
        if let Err(e) = event_queue.dispatch_pending(&mut state) {
            warn!("Failed to dispatch events: {}", e);
        }

        // Process IPC requests (non-blocking)
        // First collect all pending requests to avoid borrow conflict
        let pending_requests: Vec<_> = state
            .request_rx
            .as_ref()
            .map(|rx| {
                let mut reqs = Vec::new();
                while let Ok(request) = rx.try_recv() {
                    reqs.push(request);
                }
                reqs
            })
            .unwrap_or_default();

        for request in pending_requests {
            debug!("Processing IPC command: {:?}", request.command);
            let response = state.handle_command_with_response(request.command);
            if let Err(e) = request.response_tx.send(response) {
                warn!("Failed to send IPC response: {}", e);
            }
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

        // Apply power management periodically (not every frame)
        if last_power_check.elapsed() >= POWER_CHECK_INTERVAL {
            state.apply_power_management();
            last_power_check = std::time::Instant::now();
        }

        // Render all surfaces
        // Unlike the previous approach that relied on frame callbacks to drive rendering,
        // we now render on every loop iteration. This is simpler and more robust:
        // - mpv's render() will only do work if there's a new frame
        // - Wayland frame callbacks are used for vsync pacing
        // - The event loop's poll() provides natural rate limiting
        let egl_ctx = state.egl_context.as_ref();
        let qh = event_queue.handle();

        for surface in state.surfaces.values_mut() {
            // Only render if frame callback has triggered (vsync)
            // frame_pending is set by on_frame_ready() when compositor sends frame callback
            if !surface.has_frame_pending() {
                continue;
            }

            // Render the surface
            // render() will: clear frame_pending, render, request_frame, commit
            state.frame_timing.begin_frame();

            if let Err(e) = surface.render(egl_ctx, &qh) {
                warn!("Render error: {}", e);
            }

            state.frame_timing.end_frame();
        }

        // Flush after rendering to ensure frame callbacks are sent immediately
        // This is critical for blocking poll to receive the callback
        if let Err(e) = event_queue.flush() {
            warn!("Failed to flush after render: {}", e);
        }

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

/// Notify systemd that daemon is ready (Linux-specific)
#[cfg(target_os = "linux")]
fn notify_systemd_ready() {
    use std::os::unix::net::UnixDatagram;

    // Check if we're running under systemd
    if let Ok(notify_socket) = std::env::var("NOTIFY_SOCKET") {
        if !notify_socket.is_empty() {
            // Try to send notification via Unix socket
            if let Ok(sock) = UnixDatagram::unbound() {
                match sock.send_to(b"READY=1", &notify_socket) {
                    Ok(_) => {
                        info!("✓ Notified systemd: service ready");
                    }
                    Err(e) => {
                        warn!("Failed to notify systemd: {}", e);
                    }
                }
            }
        }
    }
}
