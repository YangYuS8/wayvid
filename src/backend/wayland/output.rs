use crate::core::types::OutputInfo;
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
}
