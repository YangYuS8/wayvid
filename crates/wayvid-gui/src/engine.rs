//! Playback engine integration for wayvid-gui
//!
//! This module manages the embedded PlaybackEngine lifecycle and provides
//! a bridge between the iced GUI and the Wayland rendering engine.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use tracing::info;
use wayvid_engine::{spawn_engine, EngineCommand, EngineConfig, EngineEvent, EngineHandle};

use crate::ipc_server::{IpcServer, SharedStatusCache};

/// Wrapper around the engine handle with convenience methods
pub struct EngineController {
    /// The actual engine handle
    handle: Option<EngineHandle>,
    /// Event receiver
    events_rx: Option<Receiver<EngineEvent>>,
    /// IPC server for wayvid-ctl communication
    ipc_server: Option<IpcServer>,
    /// Shared status cache for IPC queries
    status_cache: Option<SharedStatusCache>,
}

#[allow(dead_code)] // Methods reserved for future UI integration
impl EngineController {
    /// Create an uninitialized engine controller
    pub fn new() -> Self {
        Self {
            handle: None,
            events_rx: None,
            ipc_server: None,
            status_cache: None,
        }
    }

    /// Check if engine is running
    pub fn is_running(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.is_running())
            .unwrap_or(false)
    }

    /// Start the engine
    pub fn start(&mut self, config: EngineConfig) -> Result<(), String> {
        if self.is_running() {
            return Err("Engine is already running".to_string());
        }

        info!("Starting integrated playback engine");

        let (handle, events_rx) = spawn_engine(config).map_err(|e| e.to_string())?;

        // Start IPC server for wayvid-ctl communication
        let mut ipc_server = IpcServer::new();
        if let Err(e) = ipc_server.start(handle.command_sender()) {
            // IPC server failure is non-fatal, just log it
            tracing::warn!("Failed to start IPC server: {}", e);
        } else {
            info!("IPC server started for wayvid-ctl integration");
            self.status_cache = Some(ipc_server.status_cache());
            self.ipc_server = Some(ipc_server);
        }

        self.handle = Some(handle);
        self.events_rx = Some(events_rx);

        info!("Playback engine started");
        Ok(())
    }

    /// Stop the engine
    pub fn stop(&mut self) {
        info!("Stopping playback engine");

        // Stop IPC server first
        if let Some(mut ipc_server) = self.ipc_server.take() {
            ipc_server.stop();
            info!("IPC server stopped");
        }
        self.status_cache = None;

        if let Some(handle) = self.handle.take() {
            handle.request_shutdown();
            // Join with timeout (don't block forever)
            let _ = handle.join();
        }
        self.events_rx = None;

        info!("Playback engine stopped");
    }

    /// Poll for engine events (non-blocking) and update IPC status cache
    pub fn poll_events(&self) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        if let Some(ref rx) = self.events_rx {
            while let Ok(event) = rx.try_recv() {
                // Update IPC status cache based on events
                self.update_status_cache(&event);
                events.push(event);
            }
        }
        events
    }

    /// Update the IPC status cache based on engine events
    fn update_status_cache(&self, event: &EngineEvent) {
        if let Some(ref cache) = self.status_cache {
            if let Ok(mut cache) = cache.write() {
                match event {
                    EngineEvent::Started => {
                        cache.running = true;
                    }
                    EngineEvent::Stopped => {
                        cache.running = false;
                        cache.active_wallpapers.clear();
                    }
                    EngineEvent::WallpaperApplied { output, path } => {
                        cache.active_wallpapers.insert(output.clone(), path.clone());
                    }
                    EngineEvent::WallpaperCleared { output } => {
                        cache.active_wallpapers.remove(output);
                    }
                    _ => {}
                }
            }
        }
    }
    /// Send a command to the engine
    pub fn send_command(&self, command: EngineCommand) -> Result<(), String> {
        if let Some(ref handle) = self.handle {
            handle.send(command).map_err(|e| e.to_string())
        } else {
            Err("Engine is not running".to_string())
        }
    }

    /// Apply wallpaper to an output
    pub fn apply_wallpaper(&self, output: Option<String>, path: PathBuf) -> Result<(), String> {
        self.send_command(EngineCommand::ApplyWallpaper { output, path })
    }

    /// Clear wallpaper from an output
    pub fn clear_wallpaper(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::ClearWallpaper { output })
    }

    /// Set volume for an output
    pub fn set_volume(&self, output: String, volume: f32) -> Result<(), String> {
        self.send_command(EngineCommand::SetVolume { output, volume })
    }

    /// Pause playback on an output
    pub fn pause(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::Pause { output })
    }

    /// Resume playback on an output
    pub fn resume(&self, output: Option<String>) -> Result<(), String> {
        self.send_command(EngineCommand::Resume { output })
    }

    /// Request engine shutdown
    pub fn shutdown(&self) -> Result<(), String> {
        self.send_command(EngineCommand::Shutdown)
    }
}

impl Default for EngineController {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EngineController {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Create an iced subscription for polling engine events
pub fn engine_subscription(running: bool) -> iced::Subscription<crate::messages::Message> {
    use iced::time;
    use std::time::Duration;

    if !running {
        return iced::Subscription::none();
    }

    // Poll for engine events every 100ms
    time::every(Duration::from_millis(100)).map(|_| crate::messages::Message::PollEngineEvents)
}

/// Default engine configuration from app settings
pub fn default_engine_config(settings: &crate::settings::AppSettings) -> EngineConfig {
    use wayvid_engine::VideoConfig;

    EngineConfig {
        video: VideoConfig {
            volume: settings.playback.volume as f64,
            ..VideoConfig::default()
        },
        auto_play: true,
        fps_limit: settings.playback.fps_limit.map(|f| f as u32),
        pause_on_battery: settings.power.pause_on_battery,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_controller_creation() {
        let controller = EngineController::new();
        assert!(!controller.is_running());
    }
}
