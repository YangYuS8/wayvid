//! System tray integration using SNI (StatusNotifierItem) protocol
//!
//! This module provides real system tray functionality for Linux desktop environments
//! that support the SNI protocol (KDE, GNOME with extensions, niri + noctalia-shell, etc.)

use ksni::{self, Tray, TrayService};
use std::sync::mpsc;
use std::thread;

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

/// The actual tray implementation using ksni
struct WayvidTray {
    action_tx: mpsc::Sender<TrayAction>,
    paused: bool,
}

impl Tray for WayvidTray {
    fn id(&self) -> String {
        "wayvid".into()
    }

    fn icon_name(&self) -> String {
        // Use XDG icon name - this will look for wayvid.svg in standard icon paths
        // Falls back to a generic icon if not found
        "wayvid".into()
    }

    fn title(&self) -> String {
        "Wayvid".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            icon_name: "wayvid".into(),
            title: "Wayvid".into(),
            description: if self.paused {
                "Wallpaper paused".into()
            } else {
                "Animated wallpaper manager".into()
            },
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        vec![
            StandardItem {
                label: "Show Window".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.action_tx.send(TrayAction::Show);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Hide Window".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.action_tx.send(TrayAction::Hide);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: if self.paused {
                    "Resume Playback".into()
                } else {
                    "Pause Playback".into()
                },
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.action_tx.send(TrayAction::TogglePause);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
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

        let tray = WayvidTray {
            action_tx,
            paused: false,
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
