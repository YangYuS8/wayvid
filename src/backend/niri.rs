//! Niri compositor integration
#![allow(dead_code)] // Future API extensions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use tracing::{debug, info};

/// Niri IPC socket path
fn niri_socket_path() -> Result<PathBuf> {
    let socket =
        std::env::var("NIRI_SOCKET").context("NIRI_SOCKET not set - not running under Niri?")?;
    Ok(PathBuf::from(socket))
}

/// Niri IPC client
pub struct NiriClient {
    stream: UnixStream,
}

impl NiriClient {
    /// Connect to Niri IPC socket
    pub fn connect() -> Result<Self> {
        let socket = niri_socket_path()?;
        debug!("Connecting to Niri socket: {:?}", socket);

        let stream = UnixStream::connect(&socket)
            .with_context(|| format!("Failed to connect to Niri socket: {:?}", socket))?;

        info!("Connected to Niri IPC");
        Ok(Self { stream })
    }

    /// Send request and get response
    fn request(&mut self, request: &NiriRequest) -> Result<NiriResponse> {
        // Send request
        let json = serde_json::to_string(request)?;
        writeln!(self.stream, "{}", json)?;
        self.stream.flush()?;

        // Read response
        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let response: NiriResponse =
            serde_json::from_str(&line).context("Failed to parse Niri response")?;

        Ok(response)
    }

    /// Get current workspace info
    pub fn get_workspaces(&mut self) -> Result<Vec<Workspace>> {
        let response = self.request(&NiriRequest::Workspaces)?;
        match response {
            NiriResponse::Workspaces(workspaces) => Ok(workspaces),
            _ => Err(anyhow::anyhow!("Unexpected response type")),
        }
    }

    /// Get focused workspace ID
    pub fn get_focused_workspace(&mut self) -> Result<Option<u64>> {
        let workspaces = self.get_workspaces()?;
        Ok(workspaces.iter().find(|w| w.is_focused).map(|w| w.id))
    }

    /// Subscribe to events
    pub fn subscribe_events(&mut self) -> Result<()> {
        self.request(&NiriRequest::Subscribe {
            events: vec!["workspace".to_string()],
        })?;
        Ok(())
    }

    /// Read next event (blocking)
    pub fn read_event(&mut self) -> Result<NiriEvent> {
        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let event: NiriEvent = serde_json::from_str(&line).context("Failed to parse Niri event")?;

        Ok(event)
    }
}

/// Niri IPC request
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum NiriRequest {
    Workspaces,
    Subscribe { events: Vec<String> },
}

/// Niri IPC response
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum NiriResponse {
    Workspaces(Vec<Workspace>),
    Ok,
}

/// Workspace info
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub id: u64,
    pub name: Option<String>,
    pub is_focused: bool,
    pub is_active: bool,
}

/// Niri event
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum NiriEvent {
    WorkspaceActivated { id: u64, focused: bool },
    WorkspacesChanged,
}

/// Check if running under Niri
pub fn is_niri() -> bool {
    std::env::var("NIRI_SOCKET").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_niri() {
        // This will vary depending on environment
        let _ = is_niri();
    }

    #[test]
    fn test_socket_path_format() {
        if let Ok(path) = niri_socket_path() {
            assert!(path.to_string_lossy().contains("niri"));
        }
    }
}
