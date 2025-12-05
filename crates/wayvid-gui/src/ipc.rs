//! IPC client for communicating with wayvid daemon
//!
//! Provides Unix socket communication with the wayvid daemon for:
//! - Applying wallpapers to outputs
//! - Querying daemon status and monitor information
//! - Controlling playback (pause/resume/stop)

#![allow(dead_code)] // Many items reserved for future IPC implementation

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use iced::futures::channel::mpsc;
use iced::futures::Stream;
use iced::Subscription;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::RwLock;

use wayvid_core::ipc::{
    default_socket_path, IpcRequest, IpcResponse, OutputInfo, OutputStatus as CoreOutputStatus,
};

use crate::messages::Message;

/// Connection state for IPC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    /// Not connected to daemon
    #[default]
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and ready
    Connected,
    /// Connection error occurred
    Error,
}

/// Daemon status information (GUI-friendly version)
#[derive(Debug, Clone, Default)]
pub struct DaemonStatus {
    /// Whether daemon is running and connected
    pub running: bool,
    /// Daemon version string
    pub version: Option<String>,
    /// Per-output status
    pub outputs: Vec<OutputStatus>,
}

/// Per-output status (GUI-friendly version)
#[derive(Debug, Clone)]
pub struct OutputStatus {
    /// Output name (e.g., "eDP-1")
    pub name: String,
    /// Currently playing wallpaper path
    pub wallpaper: Option<String>,
    /// Whether playback is paused
    pub paused: bool,
    /// Current volume level (0.0 - 1.0)
    pub volume: f32,
}

impl From<CoreOutputStatus> for OutputStatus {
    fn from(status: CoreOutputStatus) -> Self {
        Self {
            name: status.name,
            wallpaper: status.wallpaper,
            paused: status.paused,
            volume: status.volume,
        }
    }
}

/// IPC client for daemon communication
#[derive(Debug)]
pub struct IpcClient {
    socket_path: PathBuf,
    state: Arc<RwLock<ConnectionState>>,
    last_error: Arc<RwLock<Option<String>>>,
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new() -> Self {
        Self {
            socket_path: default_socket_path(),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            last_error: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Get current connection state
    pub async fn connection_state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Get last error message
    pub async fn last_error(&self) -> Option<String> {
        self.last_error.read().await.clone()
    }

    /// Check if daemon socket exists (quick check without connecting)
    pub fn socket_exists(&self) -> bool {
        self.socket_path.exists()
    }

    /// Connect to the daemon and send a request
    async fn send_request(&self, request: &IpcRequest) -> Result<IpcResponse> {
        // Update state to connecting
        *self.state.write().await = ConnectionState::Connecting;

        // Connect to socket with timeout
        let stream = tokio::time::timeout(
            Duration::from_secs(5),
            UnixStream::connect(&self.socket_path),
        )
        .await
        .context("Connection timeout")?
        .context("Failed to connect to daemon socket")?;

        let (reader, mut writer) = stream.into_split();

        // Serialize and send request
        let request_json = serde_json::to_string(request).context("Failed to serialize request")?;
        writer
            .write_all(request_json.as_bytes())
            .await
            .context("Failed to send request")?;
        writer
            .write_all(b"\n")
            .await
            .context("Failed to send newline")?;
        writer.flush().await.context("Failed to flush")?;

        // Read response with timeout
        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();

        tokio::time::timeout(
            Duration::from_secs(10),
            buf_reader.read_line(&mut response_line),
        )
        .await
        .context("Response timeout")?
        .context("Failed to read response")?;

        // Parse response
        let response: IpcResponse =
            serde_json::from_str(&response_line).context("Failed to parse response")?;

        // Update state to connected
        *self.state.write().await = ConnectionState::Connected;
        *self.last_error.write().await = None;

        Ok(response)
    }

    /// Handle connection error
    async fn handle_error(&self, error: &anyhow::Error) {
        *self.state.write().await = ConnectionState::Error;
        *self.last_error.write().await = Some(error.to_string());
        tracing::warn!("IPC error: {}", error);
    }

    /// Check if daemon is running by sending a ping
    pub async fn is_running(&self) -> bool {
        if !self.socket_exists() {
            *self.state.write().await = ConnectionState::Disconnected;
            return false;
        }

        match self.send_request(&IpcRequest::Ping).await {
            Ok(IpcResponse::Pong) => true,
            Ok(_) => true, // Any response means daemon is alive
            Err(e) => {
                self.handle_error(&e).await;
                false
            }
        }
    }

    /// Get daemon status
    pub async fn status(&self) -> Result<DaemonStatus> {
        if !self.socket_exists() {
            return Ok(DaemonStatus::default());
        }

        match self.send_request(&IpcRequest::Status).await {
            Ok(IpcResponse::Status {
                running,
                version,
                outputs,
            }) => Ok(DaemonStatus {
                running,
                version,
                outputs: outputs.into_iter().map(OutputStatus::from).collect(),
            }),
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Daemon error: {}", error);
            }
            Ok(resp) => {
                anyhow::bail!("Unexpected response: {:?}", resp);
            }
            Err(e) => {
                self.handle_error(&e).await;
                Err(e)
            }
        }
    }

    /// Get list of available outputs/monitors
    pub async fn get_outputs(&self) -> Result<Vec<OutputInfo>> {
        if !self.socket_exists() {
            return Ok(vec![]);
        }

        match self.send_request(&IpcRequest::Outputs).await {
            Ok(IpcResponse::Outputs { outputs }) => Ok(outputs),
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Daemon error: {}", error);
            }
            Ok(resp) => {
                anyhow::bail!("Unexpected response: {:?}", resp);
            }
            Err(e) => {
                self.handle_error(&e).await;
                Err(e)
            }
        }
    }

    /// Apply a wallpaper to output(s)
    pub async fn apply_wallpaper(
        &self,
        path: &str,
        output: Option<&str>,
        mode: Option<&str>,
    ) -> Result<()> {
        let request = IpcRequest::Apply {
            path: PathBuf::from(path),
            output: output.map(String::from),
            mode: mode.unwrap_or("fill").to_string(),
        };

        match self.send_request(&request).await {
            Ok(IpcResponse::Ok { .. }) => Ok(()),
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Failed to apply wallpaper: {}", error);
            }
            Ok(resp) => {
                anyhow::bail!("Unexpected response: {:?}", resp);
            }
            Err(e) => {
                self.handle_error(&e).await;
                Err(e)
            }
        }
    }

    /// Pause playback on output(s)
    pub async fn pause(&self, output: Option<&str>) -> Result<()> {
        let request = IpcRequest::Pause {
            output: output.map(String::from),
        };

        match self.send_request(&request).await {
            Ok(IpcResponse::Ok { .. }) => Ok(()),
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Failed to pause: {}", error);
            }
            Ok(resp) => {
                anyhow::bail!("Unexpected response: {:?}", resp);
            }
            Err(e) => {
                self.handle_error(&e).await;
                Err(e)
            }
        }
    }

    /// Resume playback on output(s)
    pub async fn resume(&self, output: Option<&str>) -> Result<()> {
        let request = IpcRequest::Resume {
            output: output.map(String::from),
        };

        match self.send_request(&request).await {
            Ok(IpcResponse::Ok { .. }) => Ok(()),
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Failed to resume: {}", error);
            }
            Ok(resp) => {
                anyhow::bail!("Unexpected response: {:?}", resp);
            }
            Err(e) => {
                self.handle_error(&e).await;
                Err(e)
            }
        }
    }

    /// Stop playback and clear wallpaper on output(s)
    pub async fn stop(&self, output: Option<&str>) -> Result<()> {
        let request = IpcRequest::Stop {
            output: output.map(String::from),
        };

        match self.send_request(&request).await {
            Ok(IpcResponse::Ok { .. }) => Ok(()),
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Failed to stop: {}", error);
            }
            Ok(resp) => {
                anyhow::bail!("Unexpected response: {:?}", resp);
            }
            Err(e) => {
                self.handle_error(&e).await;
                Err(e)
            }
        }
    }

    /// Set volume on an output
    pub async fn set_volume(&self, output: &str, volume: f32) -> Result<()> {
        let request = IpcRequest::SetVolume {
            output: output.to_string(),
            volume: volume.clamp(0.0, 1.0),
        };

        match self.send_request(&request).await {
            Ok(IpcResponse::Ok { .. }) => Ok(()),
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Failed to set volume: {}", error);
            }
            Ok(resp) => {
                anyhow::bail!("Unexpected response: {:?}", resp);
            }
            Err(e) => {
                self.handle_error(&e).await;
                Err(e)
            }
        }
    }

    /// Request daemon to quit
    pub async fn quit(&self) -> Result<()> {
        match self.send_request(&IpcRequest::Quit).await {
            Ok(IpcResponse::Ok { .. }) => {
                *self.state.write().await = ConnectionState::Disconnected;
                Ok(())
            }
            Ok(IpcResponse::Error { error }) => {
                anyhow::bail!("Failed to quit daemon: {}", error);
            }
            Ok(_) => {
                // Daemon may close connection before responding
                *self.state.write().await = ConnectionState::Disconnected;
                Ok(())
            }
            Err(_) => {
                // Connection closed is expected when quitting
                *self.state.write().await = ConnectionState::Disconnected;
                Ok(())
            }
        }
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for IpcClient {
    fn clone(&self) -> Self {
        Self {
            socket_path: self.socket_path.clone(),
            state: Arc::clone(&self.state),
            last_error: Arc::clone(&self.last_error),
        }
    }
}

/// IPC events for the subscription
#[derive(Debug, Clone)]
pub enum IpcEvent {
    /// Connection state changed
    ConnectionChanged(ConnectionState),
    /// Daemon status received
    StatusReceived(DaemonStatus),
    /// Error occurred
    Error(String),
}

/// State for the IPC subscription worker
enum IpcWorkerState {
    Starting,
    Running(mpsc::Receiver<()>),
    #[allow(dead_code)]
    Finished,
}

/// Create an IPC subscription that polls daemon status periodically
pub fn ipc_subscription() -> Subscription<Message> {
    struct IpcSubscription;

    Subscription::run_with_id(std::any::TypeId::of::<IpcSubscription>(), ipc_stream())
}

/// Create a stream of IPC events
fn ipc_stream() -> impl Stream<Item = Message> {
    iced::futures::stream::unfold(IpcWorkerState::Starting, |state| async move {
        match state {
            IpcWorkerState::Starting => {
                // Create a channel for shutdown (not used currently, but ready for future)
                let (_tx, rx) = mpsc::channel::<()>(1);

                // Initial connection check - only if socket exists
                let client = IpcClient::new();

                // Only report connected if socket exists AND daemon is responding
                if client.socket_exists() {
                    let is_running = client.is_running().await;
                    let message = if is_running {
                        Message::IpcConnectionChanged(ConnectionState::Connected)
                    } else {
                        // Socket exists but daemon not responding - might be starting up
                        // Don't change state, just continue polling
                        Message::IpcConnectionChanged(ConnectionState::Disconnected)
                    };
                    Some((message, IpcWorkerState::Running(rx)))
                } else {
                    // No socket - v0.5 standalone mode, no external daemon needed
                    // Don't report disconnected, just stay in standalone mode
                    tracing::debug!("IPC: No daemon socket found, running in standalone mode");
                    Some((
                        Message::IpcConnectionChanged(ConnectionState::Disconnected),
                        IpcWorkerState::Running(rx),
                    ))
                }
            }
            IpcWorkerState::Running(rx) => {
                // Wait for poll interval
                tokio::time::sleep(Duration::from_secs(5)).await;

                let client = IpcClient::new();

                // Only poll if socket exists - avoids resetting engine state in standalone mode
                if !client.socket_exists() {
                    // No daemon socket, skip polling - don't change any state
                    // This allows the GUI to manage engine_running independently
                    Some((
                        Message::IpcConnectionChanged(ConnectionState::Disconnected),
                        IpcWorkerState::Running(rx),
                    ))
                } else {
                    // Socket exists, try to get status from daemon
                    match client.status().await {
                        Ok(status) => Some((
                            Message::IpcStatusReceived(status),
                            IpcWorkerState::Running(rx),
                        )),
                        Err(_) => {
                            // Socket exists but daemon error - report disconnected
                            Some((
                                Message::IpcConnectionChanged(ConnectionState::Disconnected),
                                IpcWorkerState::Running(rx),
                            ))
                        }
                    }
                }
            }
            IpcWorkerState::Finished => None,
        }
    })
}

/// Async function to apply wallpaper via IPC
pub async fn apply_wallpaper_ipc(wallpaper_path: &str, output: Option<&str>) -> Result<(), String> {
    let client = IpcClient::new();
    client
        .apply_wallpaper(wallpaper_path, output, None)
        .await
        .map_err(|e| e.to_string())
}

/// Async function to stop wallpaper on an output via IPC
pub async fn clear_wallpaper_ipc(output: &str) -> Result<(), String> {
    let client = IpcClient::new();
    client.stop(Some(output)).await.map_err(|e| e.to_string())
}

/// Parse monitor information from wlr-randr output
///
/// This provides a reliable way to detect monitors on Wayland compositors
/// that support wlr-output-management protocol (sway, hyprland, niri, etc.)
async fn get_monitors_from_wlr_randr() -> Option<Vec<crate::state::MonitorInfo>> {
    use std::process::Command;

    let output = Command::new("wlr-randr").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();
    let mut current_monitor: Option<crate::state::MonitorInfo> = None;

    for line in stdout.lines() {
        // Output name line (not indented)
        // Format: "eDP-1 \"AU Optronics 0x573D (eDP-1)\""
        if !line.starts_with(' ') && !line.is_empty() {
            // Save previous monitor if exists
            if let Some(monitor) = current_monitor.take() {
                monitors.push(monitor);
            }

            // Extract output name (first word before space or quote)
            let name = line.split_whitespace().next().unwrap_or("").to_string();
            if !name.is_empty() {
                current_monitor = Some(crate::state::MonitorInfo {
                    name,
                    width: 0,
                    height: 0,
                    x: 0,
                    y: 0,
                    scale: 1.0,
                    primary: monitors.is_empty(), // First monitor is primary
                    current_wallpaper: None,
                });
            }
        }
        // Current mode line (indented, marked with "current")
        // Format: "    2160x1440 px, 60.000000 Hz (preferred, current)"
        else if let Some(ref mut monitor) = current_monitor {
            let trimmed = line.trim();

            // Parse current resolution
            if trimmed.contains("current") && trimmed.contains(" px") {
                if let Some(resolution) = trimmed.split(" px").next() {
                    let parts: Vec<&str> = resolution.split('x').collect();
                    if parts.len() == 2 {
                        monitor.width = parts[0].trim().parse().unwrap_or(0);
                        monitor.height = parts[1].trim().parse().unwrap_or(0);
                    }
                }
            }
            // Parse position
            // Format: "  Position: 0,0"
            else if trimmed.starts_with("Position:") {
                if let Some(pos) = trimmed.strip_prefix("Position:") {
                    let pos = pos.trim();
                    let parts: Vec<&str> = pos.split(',').collect();
                    if parts.len() == 2 {
                        monitor.x = parts[0].trim().parse().unwrap_or(0);
                        monitor.y = parts[1].trim().parse().unwrap_or(0);
                    }
                }
            }
            // Parse scale
            // Format: "  Scale: 1.250000"
            else if trimmed.starts_with("Scale:") {
                if let Some(scale_str) = trimmed.strip_prefix("Scale:") {
                    monitor.scale = scale_str.trim().parse().unwrap_or(1.0);
                }
            }
        }
    }

    // Don't forget the last monitor
    if let Some(monitor) = current_monitor {
        monitors.push(monitor);
    }

    if monitors.is_empty() {
        None
    } else {
        Some(monitors)
    }
}

/// Async function to get monitors
///
/// Tries multiple methods in order:
/// 1. IPC from running engine/daemon
/// 2. wlr-randr (for wlr-based compositors: sway, hyprland, niri)
/// 3. Mock fallback data
pub async fn get_monitors_ipc() -> Vec<crate::state::MonitorInfo> {
    tracing::debug!("get_monitors_ipc: starting monitor detection");
    let client = IpcClient::new();

    // Try IPC first (only if socket exists and returns non-empty results)
    if client.socket_exists() {
        match client.get_outputs().await {
            Ok(outputs) if !outputs.is_empty() => {
                tracing::debug!("get_monitors_ipc: got {} monitors from IPC", outputs.len());
                return outputs
                    .into_iter()
                    .map(|o| crate::state::MonitorInfo {
                        name: o.name,
                        width: o.width,
                        height: o.height,
                        x: o.x,
                        y: o.y,
                        scale: 1.0, // OutputInfo doesn't have scale, default to 1.0
                        primary: o.primary,
                        current_wallpaper: None, // Will be updated from status
                    })
                    .collect();
            }
            Ok(_) => {
                tracing::debug!("IPC returned empty outputs, trying wlr-randr");
            }
            Err(e) => {
                tracing::debug!("IPC error: {}, trying wlr-randr", e);
            }
        }
    } else {
        tracing::debug!("IPC socket not available, trying wlr-randr");
    }

    // Try wlr-randr as fallback
    tracing::debug!("get_monitors_ipc: trying wlr-randr fallback");
    if let Some(monitors) = get_monitors_from_wlr_randr().await {
        tracing::info!("Detected {} monitors via wlr-randr", monitors.len());
        for m in &monitors {
            tracing::debug!(
                "  Monitor: {} ({}x{} @ {},{}, scale {})",
                m.name,
                m.width,
                m.height,
                m.x,
                m.y,
                m.scale
            );
        }
        return monitors;
    }

    tracing::warn!("Could not detect monitors, using mock data");
    // Return mock data as last resort fallback
    vec![crate::state::MonitorInfo {
        name: "eDP-1".to_string(),
        width: 1920,
        height: 1080,
        x: 0,
        y: 0,
        scale: 1.0,
        primary: true,
        current_wallpaper: None,
    }]
}

/// Start the playback engine
///
/// In v0.5 architecture, the GUI includes the integrated playback engine.
/// This function initializes the engine if not already running.
/// No external process spawning is needed.
pub async fn start_playback_engine() -> Result<(), String> {
    // In v0.5, the engine is integrated into the GUI
    // The IPC server is started automatically when the GUI starts
    // This function is now a no-op that always succeeds
    //
    // Future implementation may initialize the playback subsystem here
    // if it's not auto-started with the GUI

    let client = IpcClient::new();
    if client.is_running().await {
        return Ok(());
    }

    // Engine should auto-start with GUI, if not running there might be an issue
    // For now, we just report success as the engine initialization
    // is handled elsewhere in the GUI startup sequence
    Ok(())
}

/// Stop the playback engine
///
/// In v0.5 architecture, stopping the engine means stopping all wallpaper playback
/// but keeping the GUI running. The IPC server remains available.
pub async fn stop_playback_engine() -> Result<(), String> {
    let client = IpcClient::new();

    // If not running, nothing to do
    if !client.socket_exists() {
        return Ok(());
    }

    // Send quit command to stop playback
    // Note: This may need to be changed to a "pause all" command
    // instead of quit if we want to keep the GUI running
    client.quit().await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_client_creation() {
        let client = IpcClient::new();
        assert!(client.socket_path().to_string_lossy().contains("wayvid"));
    }

    #[test]
    fn test_connection_state_default() {
        let state = ConnectionState::default();
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[test]
    fn test_daemon_status_default() {
        let status = DaemonStatus::default();
        assert!(!status.running);
        assert!(status.version.is_none());
        assert!(status.outputs.is_empty());
    }

    #[test]
    fn test_output_status_from_core() {
        let core_status = CoreOutputStatus {
            name: "eDP-1".to_string(),
            wallpaper: Some("/path/to/wallpaper.mp4".to_string()),
            paused: false,
            volume: 0.5,
        };

        let gui_status = OutputStatus::from(core_status);
        assert_eq!(gui_status.name, "eDP-1");
        assert_eq!(
            gui_status.wallpaper,
            Some("/path/to/wallpaper.mp4".to_string())
        );
        assert!(!gui_status.paused);
        assert_eq!(gui_status.volume, 0.5);
    }
}
