//! Application settings management
//!
//! Handles loading, saving, and auto-saving of application settings.
//! Provides persistent storage for GUI preferences, playback settings,
//! autostart configuration, and power management options.

#![allow(dead_code)] // Many items reserved for future settings implementation

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};

/// Default settings file name
const SETTINGS_FILE: &str = "settings.yaml";

/// Auto-save debounce duration
const AUTO_SAVE_DEBOUNCE_MS: u64 = 500;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppSettings {
    /// GUI settings
    pub gui: GuiSettings,
    /// Playback settings
    pub playback: PlaybackSettings,
    /// Autostart settings
    pub autostart: AutostartSettings,
    /// Power management settings
    pub power: PowerSettings,
    /// Library settings
    pub library: LibrarySettings,
}

/// GUI-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GuiSettings {
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Minimize to tray on close
    pub minimize_to_tray: bool,
    /// Start minimized
    pub start_minimized: bool,
    /// Theme (dark/light/system)
    pub theme: String,
    /// Language
    pub language: String,
    /// Renderer backend (vulkan/opengl)
    pub renderer: String,
    /// Sidebar collapsed state
    pub sidebar_collapsed: bool,
    /// Detail panel visible
    pub detail_panel_visible: bool,
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            window_width: 1200,
            window_height: 800,
            minimize_to_tray: true,
            start_minimized: false,
            theme: "dark".to_string(),
            language: "system".to_string(),
            renderer: "vulkan".to_string(),
            sidebar_collapsed: false,
            detail_panel_visible: true,
        }
    }
}

/// Playback-related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackSettings {
    /// Default volume (0.0 - 1.0)
    pub volume: f32,
    /// Default FPS limit (None = unlimited)
    pub fps_limit: Option<u32>,
    /// Preferred monitor
    pub preferred_monitor: Option<String>,
    /// Loop mode
    pub loop_mode: bool,
    /// Shuffle mode
    pub shuffle: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            volume: 0.0,
            fps_limit: None,
            preferred_monitor: None,
            loop_mode: true,
            shuffle: false,
        }
    }
}

/// Autostart settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutostartSettings {
    /// Enable autostart
    pub enabled: bool,
    /// Last applied wallpaper (to restore on startup)
    pub restore_last_wallpaper: bool,
    /// Per-monitor wallpaper state
    pub monitor_states: Vec<MonitorState>,
    /// Engine was running when app closed (to restore on startup)
    pub engine_running: bool,
}

impl Default for AutostartSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            restore_last_wallpaper: true,
            monitor_states: Vec::new(),
            engine_running: false,
        }
    }
}

/// Per-monitor state for restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorState {
    /// Monitor name
    pub monitor: String,
    /// Last wallpaper path
    pub wallpaper_path: Option<PathBuf>,
}

/// Power management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerSettings {
    /// Pause on battery
    pub pause_on_battery: bool,
    /// Pause when fullscreen app is running
    pub pause_on_fullscreen: bool,
    /// Battery FPS limit (lower for power saving)
    pub battery_fps_limit: Option<u32>,
}

impl Default for PowerSettings {
    fn default() -> Self {
        Self {
            pause_on_battery: true,
            pause_on_fullscreen: true,
            battery_fps_limit: Some(15),
        }
    }
}

/// Library settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LibrarySettings {
    /// Registered wallpaper folders
    pub folders: Vec<PathBuf>,
    /// Steam Workshop enabled
    pub workshop_enabled: bool,
    /// Custom Steam library paths
    pub steam_library_paths: Vec<PathBuf>,
    /// Thumbnail size
    pub thumbnail_size: u32,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            workshop_enabled: true,
            steam_library_paths: Vec::new(),
            thumbnail_size: 256,
        }
    }
}

impl AppSettings {
    /// Get the settings file path
    pub fn settings_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("wayvid")
            .join(SETTINGS_FILE)
    }

    /// Load settings from file
    pub fn load() -> Result<Self> {
        let path = Self::settings_path();

        if !path.exists() {
            tracing::info!("Settings file not found, using defaults");
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let settings: AppSettings = serde_yaml::from_str(&content)?;

        tracing::info!("Loaded settings from {:?}", path);
        Ok(settings)
    }

    /// Save settings to file
    pub fn save(&self) -> Result<()> {
        let path = Self::settings_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        std::fs::write(&path, content)?;

        tracing::info!("Saved settings to {:?}", path);
        Ok(())
    }
}

/// Settings manager with auto-save support
pub struct SettingsManager {
    settings: Arc<RwLock<AppSettings>>,
    save_tx: mpsc::Sender<()>,
}

impl SettingsManager {
    /// Create a new settings manager
    pub async fn new() -> Result<Self> {
        let settings = Arc::new(RwLock::new(AppSettings::load()?));

        let (save_tx, mut save_rx) = mpsc::channel::<()>(16);

        // Auto-save task with debouncing
        let settings_clone = settings.clone();
        tokio::spawn(async move {
            let mut pending_save = false;

            loop {
                tokio::select! {
                    Some(()) = save_rx.recv() => {
                        pending_save = true;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(AUTO_SAVE_DEBOUNCE_MS)), if pending_save => {
                        pending_save = false;
                        let settings = settings_clone.read().await;
                        if let Err(e) = settings.save() {
                            tracing::error!("Failed to auto-save settings: {}", e);
                        }
                    }
                }
            }
        });

        Ok(Self { settings, save_tx })
    }

    /// Get a read-only reference to settings
    pub async fn get(&self) -> tokio::sync::RwLockReadGuard<'_, AppSettings> {
        self.settings.read().await
    }

    /// Update settings with a closure
    pub async fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppSettings),
    {
        {
            let mut settings = self.settings.write().await;
            f(&mut settings);
        }

        // Trigger auto-save
        self.save_tx.send(()).await.ok();
        Ok(())
    }

    /// Force immediate save
    pub async fn save_now(&self) -> Result<()> {
        let settings = self.settings.read().await;
        settings.save()
    }
}

/// Autostart manager for XDG desktop entry
pub struct AutostartManager;

impl AutostartManager {
    const DESKTOP_ENTRY: &'static str = r#"[Desktop Entry]
Type=Application
Name=Wayvid
Comment=Animated wallpaper manager for Wayland
Exec=wayvid-gui --minimized
Icon=wayvid
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
"#;

    /// Get autostart directory path
    pub fn autostart_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(".config"))
            .join("autostart")
    }

    /// Get autostart file path
    pub fn autostart_file() -> PathBuf {
        Self::autostart_dir().join("wayvid.desktop")
    }

    /// Check if autostart is enabled
    pub fn is_enabled() -> bool {
        Self::autostart_file().exists()
    }

    /// Enable autostart
    pub fn enable() -> Result<()> {
        let dir = Self::autostart_dir();
        std::fs::create_dir_all(&dir)?;

        let path = Self::autostart_file();
        std::fs::write(&path, Self::DESKTOP_ENTRY)?;

        tracing::info!("Enabled autostart at {:?}", path);
        Ok(())
    }

    /// Disable autostart
    pub fn disable() -> Result<()> {
        let path = Self::autostart_file();

        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::info!("Disabled autostart, removed {:?}", path);
        }

        Ok(())
    }

    /// Set autostart state
    pub fn set_enabled(enabled: bool) -> Result<()> {
        if enabled {
            Self::enable()
        } else {
            Self::disable()
        }
    }
}

/// Power state detection
pub struct PowerMonitor;

impl PowerMonitor {
    /// Check if running on battery
    pub fn is_on_battery() -> bool {
        // Check /sys/class/power_supply/*/status
        let power_supply = std::path::Path::new("/sys/class/power_supply");

        if let Ok(entries) = std::fs::read_dir(power_supply) {
            for entry in entries.flatten() {
                let status_path = entry.path().join("status");
                if let Ok(status) = std::fs::read_to_string(status_path) {
                    let status = status.trim().to_lowercase();
                    if status == "discharging" {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a fullscreen application is running
    /// Note: This requires compositor-specific integration
    pub fn is_fullscreen_active() -> bool {
        // TODO: Implement via Wayland protocol or compositor IPC
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();

        assert_eq!(settings.gui.window_width, 1200);
        assert_eq!(settings.gui.window_height, 800);
        assert!(settings.gui.minimize_to_tray);
        assert!(!settings.gui.start_minimized);

        assert_eq!(settings.playback.volume, 0.0);
        assert!(settings.playback.loop_mode);

        assert!(!settings.autostart.enabled);
        assert!(settings.autostart.restore_last_wallpaper);

        assert!(settings.power.pause_on_battery);
        assert!(settings.power.pause_on_fullscreen);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();

        let yaml = serde_yaml::to_string(&settings).unwrap();
        let parsed: AppSettings = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.gui.window_width, settings.gui.window_width);
        assert_eq!(parsed.playback.volume, settings.playback.volume);
    }

    #[test]
    fn test_autostart_path() {
        let path = AutostartManager::autostart_file();
        assert!(path.to_string_lossy().contains("autostart"));
        assert!(path.to_string_lossy().ends_with("wayvid.desktop"));
    }

    #[test]
    fn test_power_monitor() {
        // Just ensure it doesn't panic
        let _ = PowerMonitor::is_on_battery();
        let _ = PowerMonitor::is_fullscreen_active();
    }
}
