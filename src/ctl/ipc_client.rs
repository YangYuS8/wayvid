//! IPC client for connecting to wayvid daemon

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use tracing::{debug, info};

use super::protocol::{IpcCommand, IpcResponse};

/// IPC client for sending commands to wayvid daemon
pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    /// Connect to wayvid daemon
    pub fn connect() -> Result<Self> {
        let socket_path = Self::get_socket_path()?;

        debug!("Connecting to wayvid socket: {:?}", socket_path);

        let stream = UnixStream::connect(&socket_path).with_context(|| {
            format!(
                "❌ Cannot connect to wayvid daemon\n\n\
                 Socket path: {:?}\n\n\
                 The daemon may not be running. Please:\n\
                 1. Start the daemon: wayvid run\n\
                 2. Or check status: wayvid-ctl check\n\
                 3. View logs: journalctl --user -u wayvid -f",
                socket_path
            )
        })?;

        info!("Connected to wayvid daemon");

        Ok(Self { stream })
    }

    /// Get the socket path (same logic as server)
    fn get_socket_path() -> Result<PathBuf> {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            Ok(PathBuf::from(runtime_dir).join("wayvid.sock"))
        } else {
            let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            Ok(PathBuf::from(format!("/tmp/wayvid-{}.sock", user)))
        }
    }

    /// Send a command and receive response
    pub fn send_command(&mut self, command: &IpcCommand) -> Result<IpcResponse> {
        // Serialize and send command
        let json = serde_json::to_string(command).context("Failed to serialize command")?;

        writeln!(self.stream, "{}", json).context("Failed to write to socket")?;

        self.stream.flush().context("Failed to flush socket")?;

        // Read response
        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();

        reader
            .read_line(&mut line)
            .context("Failed to read response from socket")?;

        let response: IpcResponse =
            serde_json::from_str(&line).context("Failed to parse response")?;

        Ok(response)
    }

    /// Check if wayvid daemon is running
    pub fn is_running() -> bool {
        if let Ok(socket_path) = Self::get_socket_path() {
            socket_path.exists() && UnixStream::connect(&socket_path).is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = IpcClient::get_socket_path();
        assert!(path.is_ok());

        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("wayvid.sock"));
    }

    #[test]
    fn test_is_running() {
        // This test depends on whether wayvid is actually running
        let _ = IpcClient::is_running();
    }
}
