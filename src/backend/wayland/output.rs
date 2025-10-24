use crate::core::types::{OutputHdrCapabilities, OutputInfo};
use wayland_protocols::xdg::xdg_output::zv1::client::zxdg_output_v1;

/// Output tracking
#[derive(Debug, Clone)]
pub struct Output {
    pub info: OutputInfo,
    pub wl_output: wayland_client::protocol::wl_output::WlOutput,
    pub xdg_output: Option<zxdg_output_v1::ZxdgOutputV1>,
}

impl Output {
    pub fn new(wl_output: wayland_client::protocol::wl_output::WlOutput, name: String) -> Self {
        Self {
            info: OutputInfo {
                name,
                width: 0,
                height: 0,
                scale: 1.0,
                position: (0, 0),
                active: true,
                hdr_capabilities: OutputHdrCapabilities::default(),
            },
            wl_output,
            xdg_output: None,
        }
    }

    pub fn set_xdg_output(&mut self, xdg_output: zxdg_output_v1::ZxdgOutputV1) {
        self.xdg_output = Some(xdg_output);
    }

    pub fn update_mode(&mut self, width: i32, height: i32) {
        self.info.width = width;
        self.info.height = height;
    }

    pub fn update_scale(&mut self, scale: i32) {
        self.info.scale = scale as f64;
    }

    pub fn update_position(&mut self, x: i32, y: i32) {
        self.info.position = (x, y);
    }

    /// Query and update HDR capabilities for this output
    ///
    /// Currently returns conservative defaults (SDR only) as Wayland HDR protocols
    /// are still in development. This method is a placeholder for future HDR support
    /// when:
    /// - zwp_xx_color_management_v1 becomes stable
    /// - Hyprland HDR extensions are available
    /// - wlroots HDR support is implemented
    ///
    /// For now, we assume all outputs are SDR and rely on MPV's tone mapping
    /// for HDR content.
    #[allow(dead_code)]
    pub fn query_hdr_capabilities(&mut self) {
        // TODO: Implement actual HDR capability detection when protocols are available
        //
        // Future implementation might look like:
        // 1. Check for color_management protocol
        // 2. Query supported color spaces
        // 3. Query supported transfer functions
        // 4. Query luminance ranges
        //
        // For now, keep conservative defaults (SDR only)

        use tracing::debug;

        debug!(
            "HDR capabilities for output {}: SDR (default)",
            self.info.name
        );

        // Keep default SDR capabilities
        // self.info.hdr_capabilities = OutputHdrCapabilities::default();
    }
}
