//! Single instance detection and communication
//!
//! Ensures only one instance of wayvid-gui runs at a time.
//! If another instance is detected, sends a ShowWindow request to bring it to front.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use wayvid_core::ipc::{default_socket_path, IpcRequest, IpcResponse};

/// Check if another instance of wayvid-gui is running
///
/// Returns true if a running instance responds to ping
pub fn is_another_instance_running() -> bool {
    let socket_path = default_socket_path();

    // Try to connect to the socket
    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            // Set timeout to avoid hanging
            let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

            // Send ping request
            let request = IpcRequest::Ping;
            if let Ok(json) = serde_json::to_string(&request) {
                if stream.write_all(format!("{}\n", json).as_bytes()).is_ok() {
                    // Try to read response
                    let mut reader = BufReader::new(&stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_ok() {
                        if let Ok(response) = serde_json::from_str::<IpcResponse>(&line) {
                            return matches!(response, IpcResponse::Pong);
                        }
                    }
                }
            }
            false
        }
        Err(_) => false,
    }
}

/// Request an existing instance to show its window
///
/// Returns true if the request was successfully sent
pub fn request_show_window() -> bool {
    let socket_path = default_socket_path();

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

            let request = IpcRequest::ShowWindow;
            if let Ok(json) = serde_json::to_string(&request) {
                if stream.write_all(format!("{}\n", json).as_bytes()).is_ok() {
                    // Wait for acknowledgment
                    let mut reader = BufReader::new(&stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_ok() {
                        return serde_json::from_str::<IpcResponse>(&line)
                            .map(|r| matches!(r, IpcResponse::Ok { .. }))
                            .unwrap_or(false);
                    }
                }
            }
            false
        }
        Err(_) => false,
    }
}
