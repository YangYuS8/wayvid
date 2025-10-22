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
use crate::config::Config;
use crate::core::power::PowerManager;
use crate::video::egl::EglContext;

pub struct AppState {
    pub config: Config,
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub xdg_output_manager: Option<zxdg_output_manager_v1::ZxdgOutputManagerV1>,
    pub outputs: HashMap<u32, Output>,
    pub surfaces: HashMap<u32, WaylandSurface>,
    pub running: bool,
    pub egl_context: Option<EglContext>,
    pub power_manager: PowerManager,
    pub last_frame_time: std::time::Instant,
}

impl AppState {
    fn new(config: Config) -> Self {
        Self {
            config,
            compositor: None,
            layer_shell: None,
            xdg_output_manager: None,
            outputs: HashMap::new(),
            surfaces: HashMap::new(),
            running: true,
            egl_context: None,
            power_manager: PowerManager::new(),
            last_frame_time: std::time::Instant::now(),
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
}

// Dispatch implementations
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
                // Create surface after output is fully configured
                if let Err(e) = state.create_surface_for_output(*output_id, qh) {
                    warn!("Failed to create surface for output {}: {}", output_id, e);
                }
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

        match event {
            Event::Done { callback_data: _ } => {
                // Frame callback triggered - mark that frame is ready to render
                if let Some(surface) = state.surfaces.get_mut(output_id) {
                    surface.on_frame_ready();
                }
            }
            _ => {}
        }
    }
}

pub fn run(config: Config) -> Result<()> {
    info!("Starting wayvid Wayland backend");

    let conn = Connection::connect_to_env().context("Failed to connect to Wayland compositor")?;

    let (globals, mut event_queue) =
        registry_queue_init::<AppState>(&conn).context("Failed to initialize registry")?;

    let qh = event_queue.handle();
    let mut state = AppState::new(config);

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

    if state.compositor.is_none() {
        anyhow::bail!("wl_compositor not available");
    }
    if state.layer_shell.is_none() {
        anyhow::bail!("zwlr_layer_shell_v1 not available - compositor not supported");
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

    // Initial render
    info!("Performing initial render...");
    let egl_ctx = state.egl_context.as_ref();
    for surface in state.surfaces.values_mut() {
        if let Err(e) = surface.render(egl_ctx) {
            warn!("Initial render error: {}", e);
        }
        // Request next frame after initial render
        surface.request_frame(&qh);
    }
    info!("Initial render complete");

    // Main event loop with vsync
    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .context("Event dispatch failed")?;

        // Apply power management (battery check, pause/resume)
        state.apply_power_management();

        // Check FPS throttling
        if state.should_throttle_fps() {
            continue; // Skip rendering this frame
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
    }

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
