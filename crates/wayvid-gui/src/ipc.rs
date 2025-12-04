//! IPC client for communicating with wayvid daemon
//!
//! Sends commands to control wallpaper playback.

use std::path::PathBuf;
use anyhow::Result;

/// IPC client for daemon communication
pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new() -> Self {
        let socket_path = dirs::runtime_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("wayvid.sock");
        
        Self { socket_path }
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        self.socket_path.exists()
    }

    /// Apply a wallpaper
    pub async fn apply_wallpaper(&self, path: &str, output: Option<&str>) -> Result<()> {
        // TODO: Implement actual IPC communication
        tracing::info!("IPC: apply_wallpaper({}, {:?})", path, output);
        Ok(())
    }

    /// Pause playback
    pub async fn pause(&self) -> Result<()> {
        tracing::info!("IPC: pause");
        Ok(())
    }

    /// Resume playback
    pub async fn resume(&self) -> Result<()> {
        tracing::info!("IPC: resume");
        Ok(())
    }

    /// Stop playback
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("IPC: stop");
        Ok(())
    }

    /// Get current status
    pub async fn status(&self) -> Result<DaemonStatus> {
        // TODO: Query actual daemon status
        Ok(DaemonStatus {
            running: self.is_running().await,
            current_wallpaper: None,
            outputs: vec![],
        })
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Daemon status information
#[derive(Debug, Clone)]
pub struct DaemonStatus {
    /// Whether daemon is running
    pub running: bool,
    /// Currently playing wallpaper path
    pub current_wallpaper: Option<String>,
    /// Active outputs
    pub outputs: Vec<OutputStatus>,
}

/// Per-output status
#[derive(Debug, Clone)]
pub struct OutputStatus {
    /// Output name
    pub name: String,
    /// Wallpaper path
    pub wallpaper: Option<String>,
    /// Whether paused
    pub paused: bool,
}
