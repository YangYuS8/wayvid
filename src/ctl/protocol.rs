use serde::{Deserialize, Serialize};

use crate::core::types::VideoSource;

/// IPC command from wayvid-ctl to wayvid daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "kebab-case")]
pub enum IpcCommand {
    /// Get current status of all outputs
    GetStatus,

    /// Pause playback on specific output or all outputs
    Pause {
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },

    /// Resume playback on specific output or all outputs
    Resume {
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
    },

    /// Seek to specific time (in seconds)
    Seek { output: String, time: f64 },

    /// Switch video source for specific output
    SwitchSource { output: String, source: VideoSource },

    /// Reload configuration from file
    ReloadConfig,

    /// Set playback rate (speed)
    SetPlaybackRate { output: String, rate: f64 },

    /// Set volume
    SetVolume {
        output: String,
        volume: f64, // 0.0 - 1.0
    },

    /// Toggle mute
    ToggleMute { output: String },

    /// Set layout mode
    SetLayout {
        output: String,
        layout: String, // "fill", "contain", "stretch", "cover", "centre"
    },

    /// Quit the daemon
    Quit,
}

/// IPC response from wayvid daemon to wayvid-ctl
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum IpcResponse {
    /// Command executed successfully
    Success {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },

    /// Command failed with error message
    Error { message: String },
}

/// Status information for an output (for future use)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStatus {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub playing: bool,
    pub paused: bool,
    pub current_time: f64,
    pub duration: f64,
    pub source: String,
    pub layout: String,
    pub volume: f64,
    pub muted: bool,
    pub playback_rate: f64,
}

/// Overall daemon status (for future use)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub version: String,
    pub outputs: Vec<OutputStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_pause_command() {
        let cmd = IpcCommand::Pause { output: None };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("pause"));
    }

    #[test]
    fn test_serialize_switch_source() {
        let cmd = IpcCommand::SwitchSource {
            output: "HDMI-A-1".to_string(),
            source: VideoSource::File {
                path: "/path/to/video.mp4".to_string(),
            },
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("switch-source"));
        assert!(json.contains("HDMI-A-1"));
        assert!(json.contains("/path/to/video.mp4"));
    }

    #[test]
    fn test_deserialize_command() {
        let json = r#"{"command":"pause","output":"DP-1"}"#;
        let cmd: IpcCommand = serde_json::from_str(json).unwrap();
        match cmd {
            IpcCommand::Pause { output } => {
                assert_eq!(output, Some("DP-1".to_string()));
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_response_success() {
        let resp = IpcResponse::Success { data: None };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("success"));
    }

    #[test]
    fn test_response_error() {
        let resp = IpcResponse::Error {
            message: "Output not found".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Output not found"));
    }
}
