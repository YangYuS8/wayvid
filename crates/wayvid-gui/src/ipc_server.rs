//! Built-in IPC server for wayvid-gui
//!
//! This module implements a Unix socket server that allows external tools
//! (like wayvid-ctl) to communicate with the GUI's integrated engine.

#![allow(dead_code)] // Server will be used when engine is started with IPC enabled

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use wayvid_core::ipc::{default_socket_path, IpcRequest, IpcResponse, OutputInfo, OutputStatus};
use wayvid_engine::EngineCommand;

/// Cached engine status for IPC queries
#[derive(Debug, Clone, Default)]
pub struct EngineStatusCache {
    /// Whether engine is running
    pub running: bool,
    /// Active wallpapers per output (output_name -> wallpaper_path)
    pub active_wallpapers: HashMap<String, PathBuf>,
    /// Paused outputs
    pub paused_outputs: HashMap<String, bool>,
    /// Volume per output (default 1.0)
    pub volumes: HashMap<String, f32>,
}

impl EngineStatusCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to IPC OutputStatus list
    pub fn to_output_status_list(&self) -> Vec<OutputStatus> {
        self.active_wallpapers
            .iter()
            .map(|(name, path)| OutputStatus {
                name: name.clone(),
                wallpaper: Some(path.to_string_lossy().to_string()),
                paused: self.paused_outputs.get(name).copied().unwrap_or(false),
                volume: self.volumes.get(name).copied().unwrap_or(1.0),
            })
            .collect()
    }
}

/// Shared status cache type
pub type SharedStatusCache = Arc<RwLock<EngineStatusCache>>;

/// Shared flag for show window request
pub type ShowWindowFlag = Arc<AtomicBool>;

/// IPC server for handling external requests
pub struct IpcServer {
    /// Socket path
    socket_path: PathBuf,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Server task handle
    handle: Option<tokio::task::JoinHandle<()>>,
    /// Shared status cache
    status_cache: SharedStatusCache,
    /// Flag to signal show window request
    show_window_flag: ShowWindowFlag,
}

impl IpcServer {
    /// Create a new IPC server (not started)
    pub fn new() -> Self {
        Self {
            socket_path: default_socket_path(),
            shutdown_tx: None,
            handle: None,
            status_cache: Arc::new(RwLock::new(EngineStatusCache::new())),
            show_window_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create with custom socket path
    #[allow(dead_code)]
    pub fn with_socket_path(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            shutdown_tx: None,
            handle: None,
            status_cache: Arc::new(RwLock::new(EngineStatusCache::new())),
            show_window_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a clone of the status cache for external updates
    pub fn status_cache(&self) -> SharedStatusCache {
        Arc::clone(&self.status_cache)
    }

    /// Get a clone of the show window flag
    pub fn show_window_flag(&self) -> ShowWindowFlag {
        Arc::clone(&self.show_window_flag)
    }

    /// Start the IPC server
    ///
    /// The server will listen for incoming connections and forward requests
    /// to the engine via the provided command sender.
    pub fn start(&mut self, engine_tx: calloop::channel::Sender<EngineCommand>) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        // Remove existing socket file
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove existing socket file")?;
        }

        // Ensure parent directory exists
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
        }

        let socket_path = self.socket_path.clone();
        let status_cache = Arc::clone(&self.status_cache);
        let show_window_flag = Arc::clone(&self.show_window_flag);
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        self.shutdown_tx = Some(shutdown_tx);

        // Spawn the server task
        let handle = tokio::spawn(async move {
            if let Err(e) = run_server(
                socket_path,
                engine_tx,
                status_cache,
                show_window_flag,
                shutdown_rx,
            )
            .await
            {
                error!("IPC server error: {}", e);
            }
        });

        self.handle = Some(handle);

        info!("IPC server started on {:?}", self.socket_path);
        Ok(())
    }

    /// Stop the IPC server
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            // Send shutdown signal (ignore errors if receiver dropped)
            let _ = tx.try_send(());
        }

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }

        // Clean up socket file
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        info!("IPC server stopped");
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false)
    }

    /// Get the socket path
    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

impl Default for IpcServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Run the IPC server
async fn run_server(
    socket_path: PathBuf,
    engine_tx: calloop::channel::Sender<EngineCommand>,
    status_cache: SharedStatusCache,
    show_window_flag: ShowWindowFlag,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<()> {
    let listener = UnixListener::bind(&socket_path).context("Failed to bind socket")?;

    info!("IPC server listening on {:?}", socket_path);

    loop {
        tokio::select! {
            // Check for shutdown signal
            _ = shutdown_rx.recv() => {
                info!("IPC server shutdown requested");
                break;
            }

            // Accept new connections
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        let tx = engine_tx.clone();
                        let cache = Arc::clone(&status_cache);
                        let flag = Arc::clone(&show_window_flag);
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, tx, cache, flag).await {
                                warn!("Connection handler error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Accept error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle a single client connection
async fn handle_connection(
    stream: UnixStream,
    engine_tx: calloop::channel::Sender<EngineCommand>,
    status_cache: SharedStatusCache,
    show_window_flag: ShowWindowFlag,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        debug!("Received IPC request: {}", trimmed);

        // Parse request
        let response = match serde_json::from_str::<IpcRequest>(trimmed) {
            Ok(request) => {
                handle_request(request, &engine_tx, &status_cache, &show_window_flag).await
            }
            Err(e) => IpcResponse::Error {
                error: format!("Invalid request: {}", e),
            },
        };

        // Send response
        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        line.clear();
    }

    Ok(())
}

/// Handle a single IPC request
async fn handle_request(
    request: IpcRequest,
    engine_tx: &calloop::channel::Sender<EngineCommand>,
    status_cache: &SharedStatusCache,
    show_window_flag: &ShowWindowFlag,
) -> IpcResponse {
    match request {
        IpcRequest::Apply { path, output, .. } => {
            let cmd = EngineCommand::ApplyWallpaper { path, output };
            match engine_tx.send(cmd) {
                Ok(()) => IpcResponse::Ok {
                    message: Some("Wallpaper applied".to_string()),
                },
                Err(e) => IpcResponse::Error {
                    error: format!("Failed to send command: {}", e),
                },
            }
        }

        IpcRequest::Stop { output } => {
            let cmd = EngineCommand::ClearWallpaper { output };
            match engine_tx.send(cmd) {
                Ok(()) => IpcResponse::Ok {
                    message: Some("Wallpaper cleared".to_string()),
                },
                Err(e) => IpcResponse::Error {
                    error: format!("Failed to send command: {}", e),
                },
            }
        }

        IpcRequest::Pause { output } => {
            let cmd = EngineCommand::Pause { output };
            match engine_tx.send(cmd) {
                Ok(()) => IpcResponse::Ok {
                    message: Some("Paused".to_string()),
                },
                Err(e) => IpcResponse::Error {
                    error: format!("Failed to send command: {}", e),
                },
            }
        }

        IpcRequest::Resume { output } => {
            let cmd = EngineCommand::Resume { output };
            match engine_tx.send(cmd) {
                Ok(()) => IpcResponse::Ok {
                    message: Some("Resumed".to_string()),
                },
                Err(e) => IpcResponse::Error {
                    error: format!("Failed to send command: {}", e),
                },
            }
        }

        IpcRequest::SetVolume { output, volume } => {
            let cmd = EngineCommand::SetVolume { output, volume };
            match engine_tx.send(cmd) {
                Ok(()) => IpcResponse::Ok {
                    message: Some("Volume set".to_string()),
                },
                Err(e) => IpcResponse::Error {
                    error: format!("Failed to send command: {}", e),
                },
            }
        }

        IpcRequest::Status => {
            // Get actual output status from cache
            let outputs = status_cache
                .read()
                .map(|cache| cache.to_output_status_list())
                .unwrap_or_default();

            IpcResponse::Status {
                running: true,
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                outputs,
            }
        }

        IpcRequest::Outputs => {
            // Get outputs from cache - return basic info with just the name
            let outputs = status_cache
                .read()
                .map(|cache| {
                    cache
                        .active_wallpapers
                        .keys()
                        .map(|name| OutputInfo {
                            name: name.clone(),
                            width: 0,
                            height: 0,
                            refresh: None,
                            make: None,
                            model: None,
                            primary: false,
                            x: 0,
                            y: 0,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            IpcResponse::Outputs { outputs }
        }

        IpcRequest::Quit => {
            let cmd = EngineCommand::Shutdown;
            match engine_tx.send(cmd) {
                Ok(()) => IpcResponse::Ok {
                    message: Some("Shutting down".to_string()),
                },
                Err(e) => IpcResponse::Error {
                    error: format!("Failed to send command: {}", e),
                },
            }
        }

        IpcRequest::Ping => IpcResponse::Pong,

        IpcRequest::ShowWindow => {
            // Set the flag to notify the GUI to show the window
            show_window_flag.store(true, Ordering::SeqCst);
            info!("ShowWindow request received");
            IpcResponse::Ok {
                message: Some("Window show requested".to_string()),
            }
        }

        IpcRequest::Reload => {
            // GUI doesn't support hot reload yet
            IpcResponse::Ok {
                message: Some("Reload not supported in GUI mode".to_string()),
            }
        }

        IpcRequest::GetLibrary { .. } => {
            // TODO: Implement library query
            IpcResponse::Library {
                items: Vec::new(),
                total: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_server_creation() {
        let server = IpcServer::new();
        assert!(!server.is_running());
    }
}
