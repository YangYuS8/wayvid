//! Layer shell surface management
//!
//! Handles wlr-layer-shell protocol for creating wallpaper surfaces.

use wayland_client::{
    protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    Connection, QueueHandle,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{Anchor, ZwlrLayerSurfaceV1},
};

/// Layer surface configuration
#[derive(Debug, Clone)]
pub struct LayerSurfaceConfig {
    /// Target output name (None = all outputs)
    pub output: Option<String>,
    /// Surface width (0 = match output)
    pub width: u32,
    /// Surface height (0 = match output)
    pub height: u32,
    /// Exclusive zone (-1 = ignore, 0 = auto, >0 = pixels)
    pub exclusive_zone: i32,
    /// Margin from edges
    pub margin: LayerMargin,
    /// Keyboard interactivity
    pub keyboard_interactive: bool,
}

impl Default for LayerSurfaceConfig {
    fn default() -> Self {
        Self {
            output: None,
            width: 0,
            height: 0,
            exclusive_zone: -1,
            margin: LayerMargin::default(),
            keyboard_interactive: false,
        }
    }
}

/// Surface margins
#[derive(Debug, Clone, Default)]
pub struct LayerMargin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

/// Layer surface state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceState {
    /// Surface is being created
    Pending,
    /// Surface is configured and ready
    Configured,
    /// Surface was closed
    Closed,
}

/// Manages a single layer shell surface
pub struct LayerSurface {
    /// Wayland surface
    surface: WlSurface,
    /// Layer surface protocol object
    layer_surface: ZwlrLayerSurfaceV1,
    /// Current state
    state: SurfaceState,
    /// Configured width
    width: u32,
    /// Configured height
    height: u32,
    /// Surface configuration (reserved for future use)
    #[allow(dead_code)]
    config: LayerSurfaceConfig,
}

impl LayerSurface {
    /// Create a new layer surface
    ///
    /// Note: This is a placeholder - full implementation needs proper
    /// Wayland connection and event handling setup.
    pub fn new(
        _connection: &Connection,
        _qh: &QueueHandle<()>,
        _layer_shell: &ZwlrLayerShellV1,
        _output: Option<&WlOutput>,
        _config: LayerSurfaceConfig,
    ) -> anyhow::Result<Self> {
        // TODO: Implement proper surface creation
        // This requires:
        // 1. Creating wl_surface from compositor
        // 2. Getting layer_surface from layer_shell
        // 3. Setting up configure callback
        // 4. Initial commit

        anyhow::bail!("LayerSurface::new() not yet implemented - placeholder")
    }

    /// Get current surface state
    pub fn state(&self) -> SurfaceState {
        self.state
    }

    /// Get configured dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the underlying Wayland surface
    pub fn wl_surface(&self) -> &WlSurface {
        &self.surface
    }

    /// Acknowledge a configure event
    pub fn ack_configure(&self, serial: u32) {
        self.layer_surface.ack_configure(serial);
    }

    /// Set surface size
    pub fn set_size(&self, width: u32, height: u32) {
        self.layer_surface.set_size(width, height);
    }

    /// Set anchor edges
    pub fn set_anchor(&self, anchor: Anchor) {
        self.layer_surface.set_anchor(anchor);
    }

    /// Set exclusive zone
    pub fn set_exclusive_zone(&self, zone: i32) {
        self.layer_surface.set_exclusive_zone(zone);
    }

    /// Set margins
    pub fn set_margin(&self, top: i32, right: i32, bottom: i32, left: i32) {
        self.layer_surface.set_margin(top, right, bottom, left);
    }

    /// Commit pending changes
    pub fn commit(&self) {
        self.surface.commit();
    }
}

impl Drop for LayerSurface {
    fn drop(&mut self) {
        self.layer_surface.destroy();
        self.surface.destroy();
    }
}

/// Builder for layer surfaces
pub struct LayerSurfaceBuilder {
    config: LayerSurfaceConfig,
}

impl LayerSurfaceBuilder {
    pub fn new() -> Self {
        Self {
            config: LayerSurfaceConfig::default(),
        }
    }

    pub fn output(mut self, name: impl Into<String>) -> Self {
        self.config.output = Some(name.into());
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn exclusive_zone(mut self, zone: i32) -> Self {
        self.config.exclusive_zone = zone;
        self
    }

    pub fn margin(mut self, top: i32, right: i32, bottom: i32, left: i32) -> Self {
        self.config.margin = LayerMargin {
            top,
            right,
            bottom,
            left,
        };
        self
    }

    pub fn keyboard_interactive(mut self, interactive: bool) -> Self {
        self.config.keyboard_interactive = interactive;
        self
    }

    pub fn config(&self) -> &LayerSurfaceConfig {
        &self.config
    }
}

impl Default for LayerSurfaceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
