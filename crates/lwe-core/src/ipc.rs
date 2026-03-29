//! IPC protocol types shared between wayvid daemon, GUI, and CLI
//!
//! Defines the JSON protocol for inter-process communication.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// IPC request from client to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcRequest {
    /// Ping - check if daemon is alive
    Ping,

    /// Get daemon status
    Status,

    /// List available outputs/monitors
    Outputs,

    /// Show the GUI window (for single instance support)
    ShowWindow,

    /// Apply wallpaper to output(s)
    Apply {
        /// Path to wallpaper file
        path: PathBuf,
        /// Target output (None = all outputs)
        output: Option<String>,
        /// Scale mode: fill, contain, stretch, centre
        #[serde(default = "default_mode")]
        mode: String,
    },

    /// Pause playback
    Pause {
        /// Target output (None = all)
        output: Option<String>,
    },

    /// Resume playback
    Resume {
        /// Target output (None = all)
        output: Option<String>,
    },

    /// Stop playback and clear wallpaper
    Stop {
        /// Target output (None = all)
        output: Option<String>,
    },

    /// Set volume
    SetVolume {
        /// Target output
        output: String,
        /// Volume level 0.0 - 1.0
        volume: f32,
    },

    /// Reload configuration
    Reload,

    /// Request library list (used by GUI)
    GetLibrary {
        /// Optional filter
        filter: Option<LibraryFilter>,
    },

    /// Quit daemon
    Quit,
}

fn default_mode() -> String {
    "fill".to_string()
}

/// Library filter for GetLibrary request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibraryFilter {
    /// Filter by wallpaper type
    pub wallpaper_type: Option<String>,
    /// Search query
    pub search: Option<String>,
    /// Tags to include
    pub tags: Option<Vec<String>>,
}

/// IPC response from daemon to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcResponse {
    /// Success with optional message
    Ok { message: Option<String> },

    /// Error response
    Error { error: String },

    /// Pong response to ping
    Pong,

    /// Status response
    Status {
        running: bool,
        version: Option<String>,
        outputs: Vec<OutputStatus>,
    },

    /// Outputs list response
    Outputs { outputs: Vec<OutputInfo> },

    /// Library list response
    Library {
        items: Vec<LibraryItem>,
        total: usize,
    },
}

/// Output status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStatus {
    /// Output name (e.g., "eDP-1")
    pub name: String,
    /// Currently playing wallpaper path
    pub wallpaper: Option<String>,
    /// Whether playback is paused
    pub paused: bool,
    /// Current volume level
    pub volume: f32,
}

/// Output/monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputInfo {
    /// Output name
    pub name: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Refresh rate in Hz
    pub refresh: Option<u32>,
    /// Monitor manufacturer
    pub make: Option<String>,
    /// Monitor model
    pub model: Option<String>,
    /// Whether this is the primary monitor
    pub primary: bool,
    /// Position X
    pub x: i32,
    /// Position Y
    pub y: i32,
}

/// Library item for GetLibrary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// File path
    pub path: String,
    /// Wallpaper type
    pub wallpaper_type: String,
    /// Tags
    pub tags: Vec<String>,
    /// Is favorite
    pub favorite: bool,
}

/// Socket path helper
pub fn default_socket_path() -> PathBuf {
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
    fn test_request_ping() {
        let request = IpcRequest::Ping;
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("ping"));

        let parsed: IpcRequest = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcRequest::Ping));
    }

    #[test]
    fn test_request_apply() {
        let request = IpcRequest::Apply {
            path: PathBuf::from("/home/user/wallpaper.mp4"),
            output: Some("DP-1".to_string()),
            mode: "fill".to_string(),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("apply"));
        assert!(json.contains("wallpaper.mp4"));
        assert!(json.contains("DP-1"));
    }

    #[test]
    fn test_response_ok() {
        let response = IpcResponse::Ok {
            message: Some("Wallpaper applied".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("ok"));

        let parsed: IpcResponse = serde_json::from_str(&json).unwrap();
        if let IpcResponse::Ok { message } = parsed {
            assert_eq!(message, Some("Wallpaper applied".to_string()));
        } else {
            panic!("Expected Ok response");
        }
    }

    #[test]
    fn test_response_status() {
        let response = IpcResponse::Status {
            running: true,
            version: Some("0.5.0".to_string()),
            outputs: vec![OutputStatus {
                name: "eDP-1".to_string(),
                wallpaper: Some("/home/user/bg.mp4".to_string()),
                paused: false,
                volume: 0.5,
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("status"));
        assert!(json.contains("eDP-1"));
    }

    #[test]
    fn test_socket_path() {
        let path = default_socket_path();
        assert!(path.to_string_lossy().contains("wayvid"));
    }
}
