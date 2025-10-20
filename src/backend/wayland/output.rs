use crate::core::types::OutputInfo;

/// Output tracking
#[derive(Debug, Clone)]
pub struct Output {
    pub info: OutputInfo,
    pub wl_output: wayland_client::protocol::wl_output::WlOutput,
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
        }
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
