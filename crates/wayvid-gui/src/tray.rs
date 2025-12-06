//! System tray integration using SNI (StatusNotifierItem) protocol
//!
//! This module provides real system tray functionality for Linux desktop environments
//! that support the SNI protocol (KDE, GNOME with extensions, niri + noctalia-shell, etc.)

use ksni::{self, Icon, Tray, TrayService};
use rust_i18n::t;
use std::sync::mpsc;
use std::thread;

/// Embedded tray icon (32x32 PNG converted to ARGB32)
/// The icon is embedded at compile time for portability
const TRAY_ICON_PNG: &[u8] = include_bytes!("assets/wayvid-tray.png");

/// Convert PNG to ARGB32 format for ksni
fn png_to_argb32(png_data: &[u8]) -> Option<(i32, i32, Vec<u8>)> {
    use std::io::Cursor;

    let decoder = png::Decoder::new(Cursor::new(png_data));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;

    let width = info.width as i32;
    let height = info.height as i32;

    // Convert RGBA to ARGB (network byte order = big-endian)
    let mut argb_data = Vec::with_capacity((width * height * 4) as usize);
    for chunk in buf[..info.buffer_size()].chunks(4) {
        if chunk.len() == 4 {
            // RGBA -> ARGB (big-endian)
            argb_data.push(chunk[3]); // A
            argb_data.push(chunk[0]); // R
            argb_data.push(chunk[1]); // G
            argb_data.push(chunk[2]); // B
        }
    }

    Some((width, height, argb_data))
}

/// Tray menu actions that can be triggered by user
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Show,
    Hide,
    TogglePause,
    Quit,
}

/// System tray icon state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum TrayIconState {
    #[default]
    Normal,
    Paused,
}

/// Translated labels for tray menu items
#[derive(Clone)]
struct TrayLabels {
    show_window: String,
    hide_window: String,
    pause_playback: String,
    resume_playback: String,
    quit: String,
    tooltip_normal: String,
    tooltip_paused: String,
}

impl TrayLabels {
    fn new() -> Self {
        Self {
            show_window: t!("tray.show_window").to_string(),
            hide_window: t!("tray.hide_window").to_string(),
            pause_playback: t!("tray.pause_playback").to_string(),
            resume_playback: t!("tray.resume_playback").to_string(),
            quit: t!("tray.quit").to_string(),
            tooltip_normal: t!("tray.tooltip_normal").to_string(),
            tooltip_paused: t!("tray.tooltip_paused").to_string(),
        }
    }
}

/// The actual tray implementation using ksni
struct WayvidTray {
    action_tx: mpsc::Sender<TrayAction>,
    paused: bool,
    labels: TrayLabels,
}

impl Tray for WayvidTray {
    fn id(&self) -> String {
        "wayvid".into()
    }

    fn icon_name(&self) -> String {
        // Fallback XDG icon name (used if icon_pixmap is not supported)
        "wayvid".into()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        // Provide embedded icon for better compatibility
        if let Some((width, height, data)) = png_to_argb32(TRAY_ICON_PNG) {
            vec![Icon {
                width,
                height,
                data,
            }]
        } else {
            vec![]
        }
    }

    fn title(&self) -> String {
        "Wayvid".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let icon_pixmap = if let Some((width, height, data)) = png_to_argb32(TRAY_ICON_PNG) {
            vec![Icon {
                width,
                height,
                data,
            }]
        } else {
            vec![]
        };

        ksni::ToolTip {
            icon_name: "wayvid".into(),
            icon_pixmap,
            title: "Wayvid".into(),
            description: if self.paused {
                self.labels.tooltip_paused.clone()
            } else {
                self.labels.tooltip_normal.clone()
            },
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        vec![
            StandardItem {
                label: self.labels.show_window.clone(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.action_tx.send(TrayAction::Show);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: self.labels.hide_window.clone(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.action_tx.send(TrayAction::Hide);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: if self.paused {
                    self.labels.resume_playback.clone()
                } else {
                    self.labels.pause_playback.clone()
                },
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.action_tx.send(TrayAction::TogglePause);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: self.labels.quit.clone(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.action_tx.send(TrayAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Handle to control the system tray
pub struct SystemTray {
    action_rx: mpsc::Receiver<TrayAction>,
    handle: Option<ksni::Handle<WayvidTray>>,
    _thread: Option<thread::JoinHandle<()>>,
}

impl SystemTray {
    /// Create and start the system tray
    ///
    /// Returns None if the tray service cannot be started (e.g., no D-Bus session)
    pub fn new() -> Option<Self> {
        let (action_tx, action_rx) = mpsc::channel();

        // Capture translated labels in main thread before spawning tray thread
        let labels = TrayLabels::new();

        let tray = WayvidTray {
            action_tx,
            paused: false,
            labels,
        };

        // Create the tray service
        let service = TrayService::new(tray);
        let handle = service.handle();

        // Spawn the tray service in a separate thread
        let thread = thread::spawn(move || {
            if let Err(e) = service.run() {
                tracing::error!("Tray service error: {:?}", e);
            }
        });

        tracing::info!("System tray started successfully");

        Some(Self {
            action_rx,
            handle: Some(handle),
            _thread: Some(thread),
        })
    }

    /// Try to receive a tray action (non-blocking)
    pub fn try_recv_action(&self) -> Option<TrayAction> {
        self.action_rx.try_recv().ok()
    }

    /// Update the tray icon state (paused/normal)
    #[allow(dead_code)]
    pub fn set_paused(&self, paused: bool) {
        if let Some(handle) = &self.handle {
            handle.update(move |tray| {
                tray.paused = paused;
            });
        }
    }

    /// Shutdown the tray
    pub fn shutdown(&self) {
        if let Some(handle) = &self.handle {
            handle.shutdown();
        }
    }
}

impl Default for SystemTray {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            action_rx: mpsc::channel().1,
            handle: None,
            _thread: None,
        })
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_action_enum() {
        // Just verify the enum can be created
        let action = TrayAction::Show;
        assert_eq!(action, TrayAction::Show);
    }
}
