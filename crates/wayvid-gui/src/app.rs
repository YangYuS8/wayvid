//! Main application module
//!
//! Defines the core App struct and iced integration.

use anyhow::Result;
use iced::widget::{button, column, container, row, text, Space};
use iced::window;
use iced::{Element, Font, Length, Subscription, Task, Theme};
use rust_i18n::t;

use crate::async_loader::{self, LoaderEvent};
use crate::engine::EngineController;
use crate::i18n;
use crate::ipc::{self, ConnectionState};
use crate::messages::Message;
use crate::settings::{AppSettings, AutostartManager};
use crate::state::AppState;
use crate::theme::WayvidTheme;
use crate::tray::{SystemTray, TrayAction};
use crate::views::monitors::MonitorView;
use crate::views::{self, View};

/// Main application struct
pub struct App {
    /// Application state
    state: AppState,
    /// Current theme
    theme: WayvidTheme,
    /// Monitor view state
    monitor_view: MonitorView,
    /// Pending settings save flag (for debouncing)
    settings_dirty: bool,
    /// Integrated playback engine controller
    engine: EngineController,
    /// System tray (optional - may not be available on all systems)
    tray: Option<SystemTray>,
    /// Window ID (for daemon mode - can close and reopen window)
    window_id: Option<window::Id>,
    /// Playback paused state (for tray icon)
    paused: bool,
}

impl App {
    /// Create a new application instance (daemon mode - no window initially)
    pub fn new() -> (Self, Task<Message>) {
        let (state, task) = AppState::new();

        // Apply theme from settings
        let theme = if state.app_settings.gui.theme == "light" {
            WayvidTheme::Light
        } else {
            WayvidTheme::Dark
        };

        // Initialize system tray (may fail on some systems)
        let tray = if state.app_settings.gui.minimize_to_tray {
            SystemTray::new()
        } else {
            None
        };

        if tray.is_some() {
            tracing::info!("System tray initialized successfully");
        } else if state.app_settings.gui.minimize_to_tray {
            tracing::warn!("System tray not available, minimize to tray will be disabled");
        }

        // Open initial window
        let settings = AppSettings::load().unwrap_or_default();
        let window_width = settings.gui.window_width.max(800) as f32;
        let window_height = settings.gui.window_height.max(600) as f32;

        let (window_id, open_window) = window::open(window::Settings {
            size: iced::Size::new(window_width, window_height),
            ..Default::default()
        });

        (
            Self {
                state,
                theme,
                monitor_view: MonitorView::new(),
                settings_dirty: false,
                engine: EngineController::new(),
                tray,
                window_id: Some(window_id),
                paused: false,
            },
            Task::batch([task, open_window.map(Message::WindowOpened)]),
        )
    }

    /// Trigger settings save (marks dirty for debounced save)
    fn trigger_settings_save(&mut self) {
        self.settings_dirty = true;
        // Immediate save for now (can add debouncing later)
        if let Err(e) = self.state.app_settings.save() {
            tracing::error!("Failed to save settings: {}", e);
        } else {
            self.settings_dirty = false;
        }
    }

    /// Application title (daemon mode - receives window id)
    pub fn title(&self, _window: window::Id) -> String {
        format!("{} - {}", t!("app.title"), self.state.current_view.title())
    }

    /// Handle messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::NavigateTo(view) => {
                self.state.current_view = view;
                Task::none()
            }

            // Library operations
            Message::LoadLibrary => {
                self.state.loading = true;
                Task::perform(async { load_library().await }, Message::LibraryLoaded)
            }
            Message::LibraryLoaded(result) => {
                self.state.loading = false;
                match result {
                    Ok(wallpapers) => {
                        self.state.local_wallpapers = wallpapers;
                        self.state.refresh_wallpapers();
                    }
                    Err(e) => {
                        self.state.error = Some(t!("error.load_library", error = e).to_string());
                    }
                }
                Task::none()
            }

            // Workshop operations
            Message::ScanWorkshop => {
                self.state.workshop_scanning = true;
                Task::perform(async { scan_workshop().await }, Message::WorkshopScanned)
            }
            Message::WorkshopScanned(result) => {
                self.state.workshop_scanning = false;
                match result {
                    Ok(wallpapers) => {
                        let count = wallpapers.len();
                        self.state.workshop_wallpapers = wallpapers;
                        self.state.refresh_wallpapers();
                        if count > 0 {
                            self.state.status_message =
                                Some(t!("success.workshop_found", count = count).to_string());
                        }
                    }
                    Err(e) => {
                        // Not an error if Workshop not found - just means no WE installed
                        tracing::info!("Workshop scan: {}", e);
                    }
                }
                Task::none()
            }

            // Wallpaper selection with double-click detection
            Message::SelectWallpaper(id) => {
                let now = std::time::Instant::now();
                let double_click_threshold = std::time::Duration::from_millis(400);

                // Check for double-click
                if let Some((last_time, last_id)) = &self.state.last_click {
                    if &id == last_id && now.duration_since(*last_time) < double_click_threshold {
                        // Double-click detected - apply wallpaper
                        self.state.last_click = None;
                        return self.update(Message::DoubleClickWallpaper(id));
                    }
                }

                // Single click - select wallpaper and record click time
                self.state.selected_wallpaper = Some(id.clone());
                self.state.last_click = Some((now, id));
                Task::none()
            }
            Message::DoubleClickWallpaper(id) => {
                // Apply wallpaper to target monitor (or all if none selected)
                tracing::debug!(
                    "DoubleClickWallpaper: id={}, target_monitor={:?}",
                    id,
                    self.state.target_monitor
                );
                self.state.selected_wallpaper = Some(id.clone());
                if let Some(ref target) = self.state.target_monitor {
                    self.update(Message::ApplyToMonitor(target.clone()))
                } else {
                    self.update(Message::ApplyWallpaper(id))
                }
            }
            Message::SelectTargetMonitor(monitor) => {
                tracing::info!("SelectTargetMonitor: {:?}", monitor);
                self.state.target_monitor = monitor;
                Task::none()
            }
            Message::ApplyWallpaper(id) => {
                // Find the wallpaper to get its path
                if let Some(wallpaper) = self.state.wallpapers.iter().find(|wp| wp.id == id) {
                    let path = wallpaper.source_path.clone();

                    // If engine is running, use it directly
                    if self.engine.is_running() {
                        match self.engine.apply_wallpaper(None, path) {
                            Ok(()) => {
                                tracing::info!("Applied wallpaper to all outputs via engine");
                                self.state.status_message =
                                    Some(t!("success.wallpaper_applied").to_string());
                            }
                            Err(e) => {
                                tracing::error!("Failed to apply wallpaper via engine: {}", e);
                                self.state.error =
                                    Some(t!("error.apply_wallpaper", error = e).to_string());
                            }
                        }
                        Task::none()
                    } else {
                        // Fallback to IPC for external daemon
                        Task::perform(
                            async move { apply_wallpaper(&id).await },
                            Message::WallpaperApplied,
                        )
                    }
                } else {
                    self.state.error = Some(t!("error.wallpaper_not_found").to_string());
                    Task::none()
                }
            }
            Message::WallpaperApplied(result) => {
                match result {
                    Ok(()) => {
                        self.state.status_message =
                            Some(t!("success.wallpaper_applied").to_string());
                    }
                    Err(e) => {
                        self.state.error = Some(t!("error.apply_wallpaper", error = e).to_string());
                    }
                }
                Task::none()
            }

            // Folder management
            Message::AddFolder => {
                // TODO: Open file dialog
                Task::none()
            }
            Message::RemoveFolder(path) => {
                self.state.folders.retain(|f| f.path != path);
                Task::none()
            }
            Message::ScanFolder(path) => {
                self.state.loading = true;
                Task::perform(
                    async move { scan_folder(&path).await },
                    Message::FolderScanned,
                )
            }
            Message::FolderScanned(result) => {
                self.state.loading = false;
                match result {
                    Ok(wallpapers) => {
                        // Merge new wallpapers into library
                        for wp in wallpapers {
                            if !self.state.wallpapers.iter().any(|w| w.id == wp.id) {
                                self.state.wallpapers.push(wp);
                            }
                        }
                    }
                    Err(e) => {
                        self.state.error = Some(t!("error.scan_folder", error = e).to_string());
                    }
                }
                Task::none()
            }

            // Search and filter
            Message::SearchChanged(query) => {
                self.state.search_query = query;
                Task::none()
            }
            Message::FilterChanged(filter) => {
                self.state.current_filter = filter;
                Task::none()
            }
            Message::SourceFilterChanged(source) => {
                self.state.source_filter = source;
                Task::none()
            }

            // Settings
            Message::ToggleAutostart(enabled) => {
                self.state.app_settings.autostart.enabled = enabled;
                // Update XDG autostart entry
                if let Err(e) = AutostartManager::set_enabled(enabled) {
                    self.state.error =
                        Some(t!("error.autostart", error = e.to_string()).to_string());
                } else {
                    self.trigger_settings_save();
                }
                Task::none()
            }
            Message::ToggleMinimizeToTray(enabled) => {
                self.state.app_settings.gui.minimize_to_tray = enabled;
                self.trigger_settings_save();
                Task::none()
            }
            Message::TogglePauseOnBattery(enabled) => {
                self.state.app_settings.power.pause_on_battery = enabled;
                self.trigger_settings_save();
                Task::none()
            }
            Message::TogglePauseOnFullscreen(enabled) => {
                self.state.app_settings.power.pause_on_fullscreen = enabled;
                self.trigger_settings_save();
                Task::none()
            }
            Message::VolumeChanged(volume) => {
                self.state.app_settings.playback.volume = volume;
                self.trigger_settings_save();
                Task::none()
            }
            Message::FpsLimitChanged(limit) => {
                self.state.app_settings.playback.fps_limit = limit;
                self.trigger_settings_save();
                Task::none()
            }
            Message::LanguageChanged(language) => {
                self.state.app_settings.gui.language = language.to_code().to_string();
                i18n::set_language(language);
                self.trigger_settings_save();

                // Recreate tray to update menu labels with new language
                if self.state.app_settings.gui.minimize_to_tray {
                    // Shutdown old tray
                    if let Some(ref tray) = self.tray {
                        tray.shutdown();
                    }
                    // Create new tray with updated labels
                    self.tray = SystemTray::new();
                    // Sync paused state to new tray
                    if let Some(ref tray) = self.tray {
                        tray.set_paused(self.paused);
                    }
                    tracing::info!("Tray recreated with new language");
                }

                Task::none()
            }
            Message::SaveSettings => {
                let settings = self.state.app_settings.clone();
                Task::perform(
                    async move { save_settings(&settings).await },
                    Message::SettingsSaved,
                )
            }
            Message::SettingsSaved(result) => {
                match result {
                    Ok(()) => {
                        self.state.status_message = Some(t!("success.settings_saved").to_string());
                    }
                    Err(e) => {
                        self.state.error = Some(t!("error.save_settings", error = e).to_string());
                    }
                }
                Task::none()
            }

            // Theme
            Message::ToggleTheme => {
                self.theme = match self.theme {
                    WayvidTheme::Dark => WayvidTheme::Light,
                    WayvidTheme::Light => WayvidTheme::Dark,
                };
                // 持久化主题设置
                self.state.app_settings.gui.theme = match self.theme {
                    WayvidTheme::Dark => "dark".to_string(),
                    WayvidTheme::Light => "light".to_string(),
                };
                self.trigger_settings_save();
                Task::none()
            }

            // Layout
            Message::ToggleSidebar => {
                self.state.toggle_sidebar();
                self.state.app_settings.gui.sidebar_collapsed = self.state.sidebar_collapsed;
                self.trigger_settings_save();
                Task::none()
            }
            Message::ToggleDetailPanel => {
                self.state.toggle_detail_panel();
                self.state.app_settings.gui.detail_panel_visible = self.state.detail_panel_visible;
                self.trigger_settings_save();
                Task::none()
            }

            // Renderer
            Message::ChangeRenderer(renderer) => {
                // Convert display name to internal value (lowercase for WGPU_BACKEND)
                let internal_value = match renderer.as_str() {
                    "Vulkan" => "vulkan",
                    "OpenGL" => "opengl",
                    _ => "vulkan",
                };
                self.state.app_settings.gui.renderer = internal_value.to_string();
                self.trigger_settings_save();
                // Note: Renderer change requires restart to take effect
                self.state.status_message = Some(t!("settings.restart_required").to_string());
                Task::none()
            }

            // Error handling
            Message::DismissError => {
                self.state.error = None;
                Task::none()
            }
            Message::DismissStatus => {
                self.state.status_message = None;
                Task::none()
            }

            // Daemon communication
            Message::DaemonConnected => {
                self.state.engine_running = true;
                Task::none()
            }
            Message::DaemonDisconnected => {
                self.state.engine_running = false;
                Task::none()
            }

            // Thumbnail loading
            Message::RequestThumbnails(requests) => {
                // Mark all requested thumbnails as loading
                for request in &requests {
                    self.state
                        .thumbnail_states
                        .insert(request.id.clone(), crate::state::ThumbnailState::Loading);
                    self.state.pending_thumbnails.insert(request.id.clone());
                }
                // The actual loading is handled by the subscription
                Task::none()
            }
            Message::ThumbnailLoaded(id, data) => {
                self.state.on_thumbnail_loaded(id, data);
                Task::none()
            }
            Message::ThumbnailFailed(id, error) => {
                tracing::debug!("Thumbnail failed for {}: {}", id, error);
                self.state.on_thumbnail_failed(id, error);
                Task::none()
            }
            Message::ThumbnailBatchComplete(count) => {
                tracing::debug!("Thumbnail batch complete: {} loaded", count);
                Task::none()
            }

            // Monitor operations
            Message::RefreshMonitors => {
                Task::perform(async { refresh_monitors().await }, Message::MonitorsUpdated)
            }
            Message::PollMonitorChanges => {
                // Only refresh if monitor count might have changed
                // This is a lightweight poll - actual detection happens in refresh_monitors
                Task::perform(async { refresh_monitors().await }, |new_monitors| {
                    Message::MonitorsUpdated(new_monitors)
                })
            }
            Message::MonitorsUpdated(monitors) => {
                // Only update if monitors actually changed
                if self.state.monitors.len() != monitors.len()
                    || self
                        .state
                        .monitors
                        .iter()
                        .zip(monitors.iter())
                        .any(|(a, b)| a.name != b.name)
                {
                    tracing::info!(
                        "Monitor configuration changed: {} -> {} monitors",
                        self.state.monitors.len(),
                        monitors.len()
                    );
                    self.state.monitors = monitors;
                } else {
                    // Update wallpaper status without logging
                    self.state.monitors = monitors;
                }
                Task::none()
            }
            Message::SelectMonitor(name) => {
                self.monitor_view.select_monitor(name);
                Task::none()
            }
            Message::ApplyToMonitor(output) => {
                tracing::debug!(
                    "ApplyToMonitor: output={}, engine_running={}",
                    output,
                    self.engine.is_running()
                );
                if let Some(ref id) = self.state.selected_wallpaper {
                    // Find the wallpaper to get its path
                    if let Some(wallpaper) = self.state.wallpapers.iter().find(|wp| &wp.id == id) {
                        let path = wallpaper.source_path.clone();
                        tracing::debug!("Applying wallpaper: path={:?}", path);

                        // If engine is running, use it directly
                        if self.engine.is_running() {
                            match self.engine.apply_wallpaper(Some(output.clone()), path) {
                                Ok(()) => {
                                    tracing::info!("Applied wallpaper to {} via engine", output);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to apply wallpaper via engine: {}", e);
                                    self.state.error =
                                        Some(format!("Failed to apply wallpaper: {}", e));
                                }
                            }
                            Task::none()
                        } else {
                            tracing::debug!("Engine not running, falling back to IPC");
                            // Fallback to IPC for external daemon
                            let id = id.clone();
                            Task::perform(
                                async move { apply_wallpaper_to_monitor(&id, &output).await },
                                Message::WallpaperApplied,
                            )
                        }
                    } else {
                        self.state.error = Some(t!("error.wallpaper_not_found").to_string());
                        Task::none()
                    }
                } else {
                    self.state.error = Some(t!("error.no_wallpaper").to_string());
                    Task::none()
                }
            }
            Message::ClearMonitor(output) => {
                // If engine is running, use it directly
                if self.engine.is_running() {
                    match self.engine.clear_wallpaper(Some(output.clone())) {
                        Ok(()) => {
                            tracing::info!("Cleared wallpaper from {} via engine", output);
                        }
                        Err(e) => {
                            self.state.error = Some(format!("Failed to clear wallpaper: {}", e));
                        }
                    }
                    Task::none()
                } else {
                    // Fallback to IPC for external daemon
                    Task::perform(
                        async move { clear_monitor_wallpaper(&output).await },
                        Message::WallpaperApplied,
                    )
                }
            }

            // Engine control
            Message::StartEngine => {
                // Start the integrated playback engine
                let config = crate::engine::default_engine_config(&self.state.app_settings);
                match self.engine.start(config) {
                    Ok(()) => {
                        self.state.engine_running = true;
                        self.state.status_message = Some(t!("success.engine_started").to_string());
                        tracing::info!("Integrated playback engine started");
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Failed to start engine: {}", e));
                        tracing::error!("Failed to start engine: {}", e);
                    }
                }
                Task::none()
            }
            Message::StopEngine => {
                // Stop the integrated playback engine
                self.engine.stop();
                self.state.engine_running = false;
                self.state.status_message = Some(t!("success.engine_stopped").to_string());
                tracing::info!("Integrated playback engine stopped");
                Task::none()
            }
            Message::EngineStatusUpdated(running) => {
                self.state.engine_running = running;
                self.state.ipc_state = if running {
                    ConnectionState::Connected
                } else {
                    ConnectionState::Disconnected
                };
                if running {
                    self.state.status_message = Some(t!("success.engine_started").to_string());
                } else {
                    self.state.status_message = Some(t!("success.engine_stopped").to_string());
                }
                Task::none()
            }

            // IPC communication
            Message::IpcConnectionChanged(state) => {
                // Update IPC state
                self.state.ipc_state = state;

                // Only sync engine_running with IPC state when we have an actual daemon connection
                // In standalone mode (no daemon), engine_running is managed by StartEngine/StopEngine
                // Only set engine_running = true when connected to a real daemon
                if matches!(state, ConnectionState::Connected) {
                    self.state.engine_running = true;
                }
                // Note: We intentionally don't set engine_running = false on Disconnected
                // because in v0.5 standalone mode, the GUI manages engine state independently

                Task::none()
            }
            Message::IpcStatusReceived(status) => {
                self.state.update_from_daemon_status(status);
                Task::none()
            }
            Message::IpcError(error) => {
                self.state.error = Some(error);
                self.state.ipc_state = ConnectionState::Error;
                Task::none()
            }

            // Window close handling
            Message::WindowCloseRequested => {
                if self.state.app_settings.gui.minimize_to_tray && self.tray.is_some() {
                    // 关闭窗口但保持应用运行（daemon 模式）
                    tracing::info!(
                        "Minimize to tray enabled, closing window but keeping app running"
                    );
                    if let Some(id) = self.window_id.take() {
                        window::close(id)
                    } else {
                        Task::none()
                    }
                } else {
                    // 正常退出应用程序
                    tracing::info!("Exiting application");
                    iced::exit()
                }
            }

            // Window opened (daemon mode)
            Message::WindowOpened(id) => {
                tracing::info!("Window opened: {:?}", id);
                self.window_id = Some(id);
                Task::none()
            }

            // Window closed (daemon mode)
            Message::WindowClosed(id) => {
                tracing::info!("Window closed: {:?}", id);
                if self.window_id == Some(id) {
                    self.window_id = None;
                }
                Task::none()
            }

            // Engine events (integrated engine)
            Message::PollEngineEvents => {
                // Poll events from the integrated engine
                let events = self.engine.poll_events();
                if events.is_empty() {
                    Task::none()
                } else {
                    // Process all received events
                    Task::batch(
                        events
                            .into_iter()
                            .map(|e| Task::done(Message::EngineEvent(e))),
                    )
                }
            }
            Message::EngineEvent(event) => {
                // Handle engine event
                use wayvid_engine::EngineEvent;
                match event {
                    EngineEvent::Started => {
                        self.state.engine_running = true;
                        self.state.status_message = Some(t!("success.engine_started").to_string());
                        // Request outputs when engine starts
                        if let Err(e) = self
                            .engine
                            .send_command(wayvid_engine::EngineCommand::GetOutputs)
                        {
                            tracing::warn!("Failed to request outputs: {}", e);
                        }
                    }
                    EngineEvent::Stopped => {
                        self.state.engine_running = false;
                        self.state.status_message = Some(t!("success.engine_stopped").to_string());
                    }
                    EngineEvent::WallpaperApplied { output, path } => {
                        tracing::info!("Wallpaper applied to {}: {:?}", output, path);
                        self.state.status_message =
                            Some(t!("success.wallpaper_applied").to_string());
                    }
                    EngineEvent::WallpaperCleared { output } => {
                        tracing::info!("Wallpaper cleared from {}", output);
                    }
                    EngineEvent::OutputAdded(info) => {
                        tracing::info!("Output added: {}", info.name);
                        // Update monitor list
                        self.state
                            .monitors
                            .push(crate::state::MonitorInfo::from_output_info(&info));
                    }
                    EngineEvent::OutputRemoved(name) => {
                        tracing::info!("Output removed: {}", name);
                        // Remove from monitor list
                        self.state.monitors.retain(|m| m.name != name);
                    }
                    EngineEvent::OutputsList(outputs) => {
                        tracing::info!("Received {} outputs from engine", outputs.len());
                        // Don't replace monitor list - just log it
                        // Monitor list is managed by wlr-randr polling
                    }
                    EngineEvent::Status(status) => {
                        tracing::debug!(
                            "Engine status: running={}, outputs={}",
                            status.running,
                            status.outputs.len()
                        );
                        self.state.engine_running = status.running;
                        // Don't replace monitor list - just update engine status
                        // Monitor list is managed by wlr-randr polling
                    }
                    EngineEvent::Error(err) => {
                        self.state.error = Some(err);
                    }
                }
                Task::none()
            }

            // System tray events
            Message::PollTrayEvents => {
                if let Some(ref tray) = self.tray {
                    if let Some(action) = tray.try_recv_action() {
                        return Task::done(Message::TrayAction(action));
                    }
                }
                Task::none()
            }
            Message::TrayAction(action) => {
                match action {
                    TrayAction::Show => {
                        tracing::info!("Tray: Show window");
                        // 如果窗口不存在，重新创建窗口（daemon 模式）
                        if self.window_id.is_none() {
                            let settings = AppSettings::load().unwrap_or_default();
                            let window_width = settings.gui.window_width.max(800) as f32;
                            let window_height = settings.gui.window_height.max(600) as f32;

                            let (_id, open_window) = window::open(window::Settings {
                                size: iced::Size::new(window_width, window_height),
                                ..Default::default()
                            });
                            tracing::info!("Opening new window from tray");
                            open_window.map(Message::WindowOpened)
                        } else {
                            // 窗口存在，聚焦
                            window::get_latest().and_then(window::gain_focus)
                        }
                    }
                    TrayAction::Hide => {
                        tracing::info!("Tray: Hide window");
                        // 关闭窗口但保持应用运行（daemon 模式）
                        if let Some(id) = self.window_id {
                            window::close(id)
                        } else {
                            Task::none()
                        }
                    }
                    TrayAction::TogglePause => {
                        tracing::info!("Tray: Toggle pause");
                        // Toggle pause state
                        self.paused = !self.paused;

                        // Update tray icon state
                        if let Some(ref tray) = self.tray {
                            tray.set_paused(self.paused);
                        }

                        // Send pause/resume command to engine
                        if self.engine.is_running() {
                            if self.paused {
                                if let Err(e) = self.engine.pause(None) {
                                    tracing::error!("Failed to pause engine: {}", e);
                                } else {
                                    tracing::info!("Engine paused");
                                }
                            } else if let Err(e) = self.engine.resume(None) {
                                tracing::error!("Failed to resume engine: {}", e);
                            } else {
                                tracing::info!("Engine resumed");
                            }
                        }
                        Task::none()
                    }
                    TrayAction::Quit => {
                        tracing::info!("Tray: Quit");
                        // Properly quit the application
                        if let Some(ref tray) = self.tray {
                            tray.shutdown();
                        }
                        iced::exit()
                    }
                }
            }
            Message::ShowWindow => {
                // 重新打开窗口（daemon 模式）
                if self.window_id.is_none() {
                    let settings = AppSettings::load().unwrap_or_default();
                    let window_width = settings.gui.window_width.max(800) as f32;
                    let window_height = settings.gui.window_height.max(600) as f32;

                    let (_id, open_window) = window::open(window::Settings {
                        size: iced::Size::new(window_width, window_height),
                        ..Default::default()
                    });
                    tracing::info!("Opening new window from tray");
                    open_window.map(Message::WindowOpened)
                } else {
                    // 窗口已存在，聚焦
                    window::get_latest().and_then(window::gain_focus)
                }
            }
            Message::HideWindow => {
                // 关闭窗口但保持应用运行（daemon 模式）
                if let Some(id) = self.window_id {
                    tracing::info!("Hiding window (closing)");
                    window::close(id)
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Render the view (daemon mode - receives window id)
    pub fn view(&self, _window: window::Id) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.state.current_view {
            View::Library => views::library::view(&self.state),
            View::Folders => views::folders::view(&self.state),
            View::Monitors => views::monitors::view(
                &self.state,
                &self.monitor_view,
                Message::SelectMonitor,
                Message::ApplyToMonitor,
                Message::ClearMonitor,
                Message::RefreshMonitors,
            ),
            View::Settings => views::settings::view(&self.state),
            View::About => views::about::view(&self.state),
        };

        // Main layout with sidebar
        let sidebar = self.sidebar();

        let main_content = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20);

        let layout = row![sidebar, main_content];

        // Wrap with error/status overlay if needed
        let mut result: Element<Message> = layout.into();

        if let Some(ref error) = self.state.error {
            result = self.error_overlay(result, error);
        }

        container(result)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Render the sidebar navigation (fixed, no collapse)
    fn sidebar(&self) -> Element<'_, Message> {
        let sidebar_width = 180.0;

        // Navigation button helper
        fn nav_button(label: String, view: View, current: View) -> Element<'static, Message> {
            let is_active = view == current;
            button(text(label))
                .padding(10)
                .width(Length::Fill)
                .style(if is_active {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Message::NavigateTo(view))
                .into()
        }

        let header = text(t!("app.title").to_string()).size(20);

        let nav = column![
            header,
            Space::with_height(15),
            nav_button(
                t!("nav.library").to_string(),
                View::Library,
                self.state.current_view
            ),
            nav_button(
                t!("nav.folders").to_string(),
                View::Folders,
                self.state.current_view
            ),
            nav_button(
                t!("nav.settings").to_string(),
                View::Settings,
                self.state.current_view
            ),
            nav_button(
                t!("nav.about").to_string(),
                View::About,
                self.state.current_view
            ),
        ]
        .spacing(5)
        .padding(15)
        .width(Length::Fixed(sidebar_width));

        // Status indicator and engine control at bottom
        let status = if self.state.engine_running {
            row![
                text("●").style(|_| text::Style {
                    color: Some(iced::Color::from_rgb(0.0, 0.8, 0.0))
                }),
                text(format!(" {}", t!("status.engine_running")))
            ]
        } else {
            row![
                text("●").style(|_| text::Style {
                    color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0))
                }),
                text(format!(" {}", t!("status.engine_stopped")))
            ]
        };

        // Engine control button
        let engine_button = if self.state.engine_running {
            button(text(t!("engine.stop").to_string()))
                .padding(8)
                .width(Length::Fill)
                .style(button::danger)
                .on_press(Message::StopEngine)
        } else {
            button(text(t!("engine.start").to_string()))
                .padding(8)
                .width(Length::Fill)
                .style(button::success)
                .on_press(Message::StartEngine)
        };

        let sidebar_content = column![
            nav,
            Space::with_height(Length::Fill),
            container(column![status, Space::with_height(10), engine_button].spacing(5))
                .padding(15),
        ];

        container(sidebar_content)
            .style(container::bordered_box)
            .width(Length::Fixed(sidebar_width))
            .height(Length::Fill)
            .into()
    }

    /// Error overlay
    fn error_overlay<'a>(
        &self,
        base: Element<'a, Message>,
        error: &'a str,
    ) -> Element<'a, Message> {
        let error_text = error.to_string();
        let overlay = container(
            column![
                text(t!("error.title").to_string()).size(18),
                text(error_text),
                button(text(t!("error.dismiss").to_string())).on_press(Message::DismissError),
            ]
            .spacing(10)
            .padding(20),
        )
        .style(container::bordered_box);

        // For simplicity, just show error at top
        column![overlay, base].into()
    }

    /// Get theme (daemon mode - receives window id)
    pub fn theme(&self, _window: window::Id) -> Theme {
        self.theme.into()
    }

    /// Subscriptions (background tasks)
    pub fn subscription(&self) -> Subscription<Message> {
        // Collect thumbnail requests for visible wallpapers
        // For now, request thumbnails for all filtered wallpapers (with limit)
        let thumbnail_requests: Vec<_> = self
            .state
            .filtered_wallpapers()
            .iter()
            .take(20) // Limit initial batch
            .filter_map(|wp| {
                let id = &wp.id;
                // Skip if already loaded or loading
                if self.state.thumbnails.contains_key(id)
                    || self.state.pending_thumbnails.contains(id)
                {
                    return None;
                }

                // Create request - prefer thumbnail_path for Workshop items
                let source_path = wp
                    .thumbnail_path
                    .clone()
                    .unwrap_or_else(|| wp.source_path.clone());

                Some(async_loader::ThumbnailRequest {
                    id: id.clone(),
                    path: source_path,
                    width: 256,
                    height: 144,
                })
            })
            .collect();

        // IPC subscription for daemon status polling
        let ipc_sub = ipc::ipc_subscription();

        // Thumbnail subscription (only if there are requests)
        let thumbnail_sub = if !thumbnail_requests.is_empty() {
            let cache_dir = self.state.async_loader.cache_dir().clone();
            async_loader::thumbnail_subscription(thumbnail_requests, cache_dir, |event| match event
            {
                LoaderEvent::ThumbnailLoaded(id, data) => Message::ThumbnailLoaded(id, data),
                LoaderEvent::ThumbnailFailed(id, error) => Message::ThumbnailFailed(id, error),
                LoaderEvent::BatchComplete(count) => Message::ThumbnailBatchComplete(count),
                LoaderEvent::LibraryLoaded(result) => Message::LibraryLoaded(result),
            })
        } else {
            Subscription::none()
        };

        // Engine event polling subscription (when engine is running)
        let engine_sub = crate::engine::engine_subscription(self.engine.is_running());

        // Window close request subscription (for minimize to tray)
        let close_request_sub = window::close_requests().map(|_id| Message::WindowCloseRequested);

        // Window close events subscription (daemon mode - track when windows are closed)
        let close_events_sub = window::close_events().map(Message::WindowClosed);

        // System tray event polling (every 100ms when tray is active)
        let tray_sub = if self.tray.is_some() {
            iced::time::every(std::time::Duration::from_millis(100))
                .map(|_| Message::PollTrayEvents)
        } else {
            Subscription::none()
        };

        // Monitor change detection subscription (poll every 5 seconds)
        let monitor_poll_sub = iced::time::every(std::time::Duration::from_secs(5))
            .map(|_| Message::PollMonitorChanges);

        Subscription::batch([
            ipc_sub,
            thumbnail_sub,
            engine_sub,
            close_request_sub,
            close_events_sub,
            tray_sub,
            monitor_poll_sub,
        ])
    }
}

/// Run the application
pub fn run() -> Result<()> {
    // Load settings to get renderer preference
    let settings = AppSettings::load().unwrap_or_default();

    // Set WGPU_BACKEND environment variable based on renderer setting
    // This must be done before iced initializes wgpu
    let backend = match settings.gui.renderer.to_lowercase().as_str() {
        "vulkan" => "vulkan",
        "opengl" | "gl" => "gl",
        _ => "vulkan", // Default to Vulkan
    };

    // Only set if not already set by user
    if std::env::var("WGPU_BACKEND").is_err() {
        std::env::set_var("WGPU_BACKEND", backend);
        tracing::info!("Set WGPU_BACKEND to: {}", backend);
    } else {
        tracing::info!("WGPU_BACKEND already set, respecting user override");
    }

    // Use Noto Sans CJK SC for CJK character support (system font)
    let cjk_font = Font::with_name("Noto Sans CJK SC");

    // Use daemon mode so closing window doesn't exit app (for tray support)
    iced::daemon(App::title, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .default_font(cjk_font)
        .run_with(App::new)?;

    Ok(())
}

// Async helper functions
async fn load_library() -> Result<Vec<wayvid_core::WallpaperItem>, String> {
    // TODO: Load from wayvid-library database
    Ok(vec![])
}

async fn apply_wallpaper(id: &str) -> Result<(), String> {
    // Use IPC to apply wallpaper to all outputs
    ipc::apply_wallpaper_ipc(id, None).await
}

async fn scan_folder(path: &std::path::Path) -> Result<Vec<wayvid_core::WallpaperItem>, String> {
    use wayvid_library::FolderScanner;

    let scanner = FolderScanner::new();
    scanner.scan_folder(path, true).map_err(|e| e.to_string())
}

async fn save_settings(settings: &AppSettings) -> Result<(), String> {
    settings.save().map_err(|e| e.to_string())
}

async fn refresh_monitors() -> Vec<crate::state::MonitorInfo> {
    // Query monitors from daemon via IPC
    ipc::get_monitors_ipc().await
}

async fn apply_wallpaper_to_monitor(id: &str, output: &str) -> Result<(), String> {
    // Use IPC to apply wallpaper to specific output
    ipc::apply_wallpaper_ipc(id, Some(output)).await
}

async fn clear_monitor_wallpaper(output: &str) -> Result<(), String> {
    // Use IPC to clear wallpaper from specific output
    ipc::clear_wallpaper_ipc(output).await
}

/// Scan Steam Workshop for Wallpaper Engine wallpapers
async fn scan_workshop() -> Result<Vec<wayvid_core::WallpaperItem>, String> {
    use wayvid_library::WorkshopScanner;

    let mut scanner = WorkshopScanner::discover().map_err(|e| e.to_string())?;

    scanner.scan_all().map_err(|e| e.to_string())
}
