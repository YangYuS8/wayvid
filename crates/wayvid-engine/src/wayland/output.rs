//! Wayland output management
//!
//! Tracks available outputs and their properties.

use std::collections::HashMap;

use wayland_client::protocol::wl_output::WlOutput;
use wayvid_core::{OutputInfo, OutputHdrCapabilities};

/// Output manager for tracking Wayland outputs
pub struct OutputManager {
    /// Known outputs by name
    outputs: HashMap<String, OutputState>,
}

/// State of a single output
pub struct OutputState {
    /// Wayland output object
    pub wl_output: WlOutput,
    /// Output information
    pub info: OutputInfo,
    /// Whether output is ready (all info received)
    pub ready: bool,
}

impl OutputManager {
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    /// Register a new output
    pub fn add_output(&mut self, name: String, wl_output: WlOutput) {
        let state = OutputState {
            wl_output,
            info: OutputInfo {
                name: name.clone(),
                width: 0,
                height: 0,
                scale: 1.0,
                position: (0, 0),
                active: true,
                hdr_capabilities: OutputHdrCapabilities::default(),
            },
            ready: false,
        };
        self.outputs.insert(name, state);
    }

    /// Remove an output
    pub fn remove_output(&mut self, name: &str) -> Option<OutputState> {
        self.outputs.remove(name)
    }

    /// Update output geometry
    pub fn update_geometry(
        &mut self,
        name: &str,
        x: i32,
        y: i32,
    ) {
        if let Some(state) = self.outputs.get_mut(name) {
            state.info.position = (x, y);
        }
    }

    /// Update output mode
    pub fn update_mode(
        &mut self,
        name: &str,
        width: i32,
        height: i32,
    ) {
        if let Some(state) = self.outputs.get_mut(name) {
            state.info.width = width;
            state.info.height = height;
        }
    }

    /// Update output scale
    pub fn update_scale(&mut self, name: &str, scale: i32) {
        if let Some(state) = self.outputs.get_mut(name) {
            state.info.scale = scale as f64;
        }
    }

    /// Mark output as ready (done event received)
    pub fn mark_ready(&mut self, name: &str) {
        if let Some(state) = self.outputs.get_mut(name) {
            state.ready = true;
        }
    }

    /// Get output by name
    pub fn get(&self, name: &str) -> Option<&OutputState> {
        self.outputs.get(name)
    }

    /// Get mutable output by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut OutputState> {
        self.outputs.get_mut(name)
    }

    /// Get all output names
    pub fn output_names(&self) -> impl Iterator<Item = &str> {
        self.outputs.keys().map(|s| s.as_str())
    }

    /// Get all ready outputs
    pub fn ready_outputs(&self) -> impl Iterator<Item = (&str, &OutputInfo)> {
        self.outputs
            .iter()
            .filter(|(_, state)| state.ready)
            .map(|(name, state)| (name.as_str(), &state.info))
    }

    /// Get output count
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    /// Find output containing a point (for multi-monitor layouts)
    pub fn output_at(&self, _x: i32, _y: i32) -> Option<&str> {
        // TODO: Implement proper hit testing with output positions
        // For now, return the first output
        self.outputs.keys().next().map(|s| s.as_str())
    }
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: WlOutput requires Wayland connection, so we can't test
    // add_output without mocking. Test the basic structure instead.

    #[test]
    fn test_output_manager_new() {
        let manager = OutputManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }
}
