use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tracing::{debug, error, info, warn};

use crate::ctl::protocol::{IpcCommand, IpcResponse};

/// IPC server for receiving commands from wayvid-ctl
pub struct IpcServer {
    socket_path: PathBuf,
    _listener_thread: thread::JoinHandle<()>,
}

impl IpcServer {
    /// Create and start IPC server
    pub fn start() -> Result<(Self, Receiver<IpcCommand>)> {
        let socket_path = Self::get_socket_path()?;

        // Remove old socket if exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)
                .with_context(|| format!("Failed to remove old socket: {:?}", socket_path))?;
        }

        let listener = UnixListener::bind(&socket_path)
            .with_context(|| format!("Failed to bind Unix socket: {:?}", socket_path))?;

        info!("IPC server listening on: {:?}", socket_path);

        let (cmd_tx, cmd_rx) = channel();
        let socket_path_clone = socket_path.clone();

        let listener_thread = thread::spawn(move || {
            Self::listener_loop(listener, cmd_tx);
        });

        Ok((
            Self {
                socket_path: socket_path_clone,
                _listener_thread: listener_thread,
            },
            cmd_rx,
        ))
    }

    /// Get the socket path based on XDG_RUNTIME_DIR or fallback to /tmp
    fn get_socket_path() -> Result<PathBuf> {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            Ok(PathBuf::from(runtime_dir).join("wayvid.sock"))
        } else {
            let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            Ok(PathBuf::from(format!("/tmp/wayvid-{}.sock", user)))
        }
    }

    /// Main listener loop
    fn listener_loop(listener: UnixListener, cmd_tx: Sender<IpcCommand>) {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tx = cmd_tx.clone();
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, tx) {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Handle a single client connection
    fn handle_client(mut stream: UnixStream, cmd_tx: Sender<IpcCommand>) -> Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut line = String::new();

        reader.read_line(&mut line)?;

        debug!("Received command: {}", line.trim());

        // Parse command
        let command: IpcCommand = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse command: {}", line))?;

        // Send command to main thread
        cmd_tx
            .send(command.clone())
            .context("Failed to send command to main thread")?;

        // Send success response
        let response = IpcResponse::Success { data: None };
        let response_json = serde_json::to_string(&response)?;
        writeln!(stream, "{}", response_json)?;
        stream.flush()?;

        Ok(())
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Clean up socket file
        if self.socket_path.exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                warn!("Failed to remove socket file: {}", e);
            }
        }
    }
}

/// Send a command to the wayvid daemon and get response
pub fn send_command(command: &IpcCommand) -> Result<IpcResponse> {
    let socket_path = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("wayvid.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        PathBuf::from(format!("/tmp/wayvid-{}.sock", user))
    };

    let mut stream = UnixStream::connect(&socket_path)
        .with_context(|| format!("Failed to connect to daemon at {:?}", socket_path))?;

    // Send command
    let command_json = serde_json::to_string(command)?;
    writeln!(stream, "{}", command_json)?;
    stream.flush()?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let response: IpcResponse = serde_json::from_str(&line)
        .with_context(|| format!("Failed to parse response: {}", line))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = IpcServer::get_socket_path().unwrap();
        assert!(path.to_str().unwrap().contains("wayvid"));
    }
}
