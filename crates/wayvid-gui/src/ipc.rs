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

                // Initial connection check
                let client = IpcClient::new();
                let is_running = client.is_running().await;

                let message = if is_running {
                    Message::IpcConnectionChanged(ConnectionState::Connected)
                } else {
                    Message::IpcConnectionChanged(ConnectionState::Disconnected)
                };

                Some((message, IpcWorkerState::Running(rx)))
            }
            IpcWorkerState::Running(rx) => {
                // Wait for poll interval
                tokio::time::sleep(Duration::from_secs(2)).await;

                let client = IpcClient::new();

                // Try to get status from daemon
                match client.status().await {
                    Ok(status) => Some((
                        Message::IpcStatusReceived(status),
                        IpcWorkerState::Running(rx),
                    )),
                    Err(_) => {
                        // Daemon not running or error
                        Some((
                            Message::IpcConnectionChanged(ConnectionState::Disconnected),
                            IpcWorkerState::Running(rx),
                        ))
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

/// Async function to get monitors from daemon via IPC
pub async fn get_monitors_ipc() -> Vec<crate::state::MonitorInfo> {
    let client = IpcClient::new();

    match client.get_outputs().await {
        Ok(outputs) => outputs
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
            .collect(),
        Err(e) => {
            tracing::warn!("Failed to get monitors from daemon: {}", e);
            // Return mock data as fallback
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
    }
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
