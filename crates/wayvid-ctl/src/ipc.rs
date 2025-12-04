//! IPC client for communicating with wayvid daemon
//!
//! Uses Unix domain sockets for local communication.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// IPC request types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// Apply wallpaper
    Apply {
        path: PathBuf,
        output: Option<String>,
        mode: String,
    },
    /// Pause playback
    Pause { output: Option<String> },
    /// Resume playback
    Resume { output: Option<String> },
    /// Stop playback
    Stop { output: Option<String> },
    /// Get status
    Status,
    /// List outputs
    Outputs,
    /// Reload configuration
    Reload,
    /// Ping daemon
    Ping,
}

/// IPC response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// Success response
    Ok { message: Option<String> },
    /// Error response
    Error { error: String },
    /// Status response
    Status {
        running: bool,
        outputs: Vec<OutputStatus>,
        version: Option<String>,
    },
    /// Outputs list response
    Outputs { outputs: Vec<OutputInfo> },
    /// Pong response
    Pong,
}

/// Status of an output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStatus {
    pub name: String,
    pub wallpaper: Option<String>,
    pub paused: bool,
}

/// Information about an output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh: Option<u32>,
    pub make: Option<String>,
    pub model: Option<String>,
}

/// IPC client
pub struct IpcClient {
    socket_path: PathBuf,
    timeout: Duration,
}

impl IpcClient {
    /// Create a new IPC client
    pub fn new() -> Self {
        let socket_path = get_socket_path();
        Self {
            socket_path,
            timeout: Duration::from_secs(5),
        }
    }

    /// Create client with custom socket path
    #[allow(dead_code)]
    pub fn with_socket(path: PathBuf) -> Self {
        Self {
            socket_path: path,
            timeout: Duration::from_secs(5),
        }
    }

    /// Send a request and receive response
    fn send(&self, request: Request) -> Result<Response> {
        // Connect to socket
        let mut stream = UnixStream::connect(&self.socket_path).context(
            "Failed to connect to wayvid daemon. Is it running?\n\
             Try: wayvid-ctl ping",
        )?;

        stream
            .set_read_timeout(Some(self.timeout))
            .context("Failed to set read timeout")?;
        stream
            .set_write_timeout(Some(self.timeout))
            .context("Failed to set write timeout")?;

        // Serialize and send request
        let request_json = serde_json::to_string(&request)?;
        writeln!(stream, "{}", request_json).context("Failed to send request")?;
        stream.flush()?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .context("Failed to read response")?;

        // Parse response
        let response: Response =
            serde_json::from_str(&response_line).context("Invalid response from daemon")?;

        Ok(response)
    }

    /// Apply a wallpaper
    pub fn apply(&self, path: &Path, output: Option<&str>, mode: &str) -> Result<Response> {
        self.send(Request::Apply {
            path: path.to_path_buf(),
            output: output.map(String::from),
            mode: mode.to_string(),
        })
    }

    /// Pause playback
    pub fn pause(&self, output: Option<&str>) -> Result<Response> {
        self.send(Request::Pause {
            output: output.map(String::from),
        })
    }

    /// Resume playback
    pub fn resume(&self, output: Option<&str>) -> Result<Response> {
        self.send(Request::Resume {
            output: output.map(String::from),
        })
    }

    /// Stop playback
    pub fn stop(&self, output: Option<&str>) -> Result<Response> {
        self.send(Request::Stop {
            output: output.map(String::from),
        })
    }

    /// Get daemon status
    pub fn status(&self) -> Result<Response> {
        self.send(Request::Status)
    }

    /// List available outputs
    pub fn outputs(&self) -> Result<Response> {
        self.send(Request::Outputs)
    }

    /// Reload configuration
    pub fn reload(&self) -> Result<Response> {
        self.send(Request::Reload)
    }

    /// Ping daemon
    pub fn ping(&self) -> Result<Response> {
        self.send(Request::Ping)
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the default socket path
fn get_socket_path() -> PathBuf {
    // Try XDG_RUNTIME_DIR first (standard for user sockets)
    if let Some(runtime_dir) = dirs::runtime_dir() {
        return runtime_dir.join("wayvid.sock");
    }

    // Fallback to /tmp with username for uniqueness
    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    PathBuf::from(format!("/tmp/wayvid-{}.sock", username))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = Request::Apply {
            path: PathBuf::from("/home/user/wallpaper.mp4"),
            output: Some("DP-1".to_string()),
            mode: "fill".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("apply"));
        assert!(json.contains("wallpaper.mp4"));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{"type": "ok", "message": "Wallpaper applied"}"#;
        let response: Response = serde_json::from_str(json).unwrap();

        if let Response::Ok { message } = response {
            assert_eq!(message, Some("Wallpaper applied".to_string()));
        } else {
            panic!("Expected Ok response");
        }
    }

    #[test]
    fn test_socket_path() {
        let path = get_socket_path();
        assert!(path.to_string_lossy().contains("wayvid"));
    }
}
