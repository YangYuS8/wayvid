use anyhow::{Context, Result};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_compositor, wl_output, wl_registry, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::backend::wayland::output::Output;
use crate::backend::wayland::surface::WaylandSurface;
use crate::config::Config;

pub struct AppState {
    pub config: Config,
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub outputs: HashMap<u32, Output>,
    pub surfaces: HashMap<u32, WaylandSurface>,
    pub running: bool,
}

impl AppState {
    fn new(config: Config) -> Self {
        Self {
            config,
            compositor: None,
            layer_shell: None,
            outputs: HashMap::new(),
            surfaces: HashMap::new(),
            running: true,
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
            output.info.clone(),
            effective_config,
            &output.wl_output,
            qh,
        )?;

        info!("Created surface for output: {}", output.info.name);
        self.surfaces.insert(output_id, surface);

        Ok(())
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
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            debug!("Registry: {} v{} ({})", interface, version, name);

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
                    let layer_shell = registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(
                        name,
                        version.min(4),
                        qh,
                        (),
                    );
                    state.layer_shell = Some(layer_shell);
                    info!("Bound zwlr_layer_shell_v1");
                }
                "wl_output" => {
                    let wl_output =
                        registry.bind::<wl_output::WlOutput, _, _>(name, version.min(3), qh, name);

                    let output = Output::new(wl_output, format!("output-{}", name));
                    state.outputs.insert(name, output);
                    info!("Added output: {}", name);
                }
                _ => {}
            }
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
                for surface in state.surfaces.values_mut() {
                    if surface.layer_surface == *layer_surface {
                        surface.configure(width, height, serial);
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

pub fn run(config: Config) -> Result<()> {
    info!("Starting wayvid Wayland backend");

    let conn = Connection::connect_to_env().context("Failed to connect to Wayland compositor")?;

    let (_globals, mut event_queue) =
        registry_queue_init::<AppState>(&conn).context("Failed to initialize registry")?;

    let _qh = event_queue.handle();
    let mut state = AppState::new(config);

    // Initial roundtrip to discover globals
    event_queue
        .roundtrip(&mut state)
        .context("Initial roundtrip failed")?;

    info!("Discovered {} outputs", state.outputs.len());

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

    // Main event loop
    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .context("Event dispatch failed")?;

        // Render all surfaces
        for surface in state.surfaces.values_mut() {
            if let Err(e) = surface.render() {
                warn!("Render error: {}", e);
            }
        }
    }

    info!("Shutting down");
    Ok(())
}
