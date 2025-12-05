//! Main application module
//!
//! Defines the core App struct and iced integration.

use anyhow::Result;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Font, Length, Subscription, Task, Theme};
use rust_i18n::t;

use crate::async_loader::{self, LoaderEvent};
use crate::i18n;
use crate::ipc::{self, ConnectionState};
use crate::messages::Message;
use crate::settings::{AppSettings, AutostartManager};
use crate::state::AppState;
use crate::theme::WayvidTheme;
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
}

impl App {
    /// Create a new application instance
    pub fn new() -> (Self, Task<Message>) {
        let (state, task) = AppState::new();

        // Apply theme from settings
        let theme = if state.app_settings.gui.theme == "light" {
            WayvidTheme::Light
        } else {
            WayvidTheme::Dark
        };

        (
            Self {
                state,
                theme,
                monitor_view: MonitorView::new(),
                settings_dirty: false,
            },
            task,
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

    /// Application title
    pub fn title(&self) -> String {
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

            // Wallpaper selection
            Message::SelectWallpaper(id) => {
                self.state.selected_wallpaper = Some(id);
                Task::none()
            }
            Message::ApplyWallpaper(id) => Task::perform(
                async move { apply_wallpaper(&id).await },
                Message::WallpaperApplied,
            ),
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
                self.state.app_settings.gui.renderer = renderer;
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
            Message::MonitorsUpdated(monitors) => {
                self.state.monitors = monitors;
                Task::none()
            }
            Message::SelectMonitor(name) => {
                self.monitor_view.select_monitor(name);
                Task::none()
            }
            Message::ApplyToMonitor(output) => {
                if let Some(ref id) = self.state.selected_wallpaper {
                    let id = id.clone();
                    Task::perform(
                        async move { apply_wallpaper_to_monitor(&id, &output).await },
                        Message::WallpaperApplied,
                    )
                } else {
                    self.state.error = Some(t!("error.no_wallpaper").to_string());
                    Task::none()
                }
            }
            Message::ClearMonitor(output) => Task::perform(
                async move { clear_monitor_wallpaper(&output).await },
                Message::WallpaperApplied,
            ),

            // Engine control
            Message::StartEngine => {
                Task::perform(async { ipc::start_playback_engine().await }, |result| {
                    Message::EngineStatusUpdated(result.is_ok())
                })
            }
            Message::StopEngine => {
                Task::perform(async { ipc::stop_playback_engine().await }, |result| {
                    Message::EngineStatusUpdated(result.is_err())
                })
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
        }
    }

    /// Render the view
    pub fn view(&self) -> Element<'_, Message> {
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

        // Monitor selector section (at bottom of sidebar)
        let monitor_selector: Element<Message> = if self.state.monitors.is_empty() {
            text(t!("monitors.no_monitors").to_string()).size(12).into()
        } else {
            let monitor_buttons: Vec<Element<Message>> = self
                .state
                .monitors
                .iter()
                .map(|m| {
                    let is_selected = self.monitor_view.selected_monitor() == Some(m.name.as_str());
                    let label = format!("{} ({}x{})", m.name, m.width, m.height);
                    button(text(label).size(11))
                        .padding(6)
                        .width(Length::Fill)
                        .style(if is_selected {
                            button::primary
                        } else {
                            button::secondary
                        })
                        .on_press(Message::SelectMonitor(m.name.clone()))
                        .into()
                })
                .collect();

            column(monitor_buttons).spacing(4).into()
        };

        let monitor_section = column![
            text(t!("nav.monitors").to_string()).size(14),
            Space::with_height(5),
            monitor_selector,
        ]
        .spacing(3)
        .padding(10);

        let sidebar_content = column![
            nav,
            Space::with_height(Length::Fill),
            container(monitor_section)
                .style(container::bordered_box)
                .width(Length::Fill),
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

    /// Get theme
    pub fn theme(&self) -> Theme {
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

        Subscription::batch([ipc_sub, thumbnail_sub])
    }
}

/// Run the application
pub fn run() -> Result<()> {
    // Use Noto Sans CJK SC for CJK character support (system font)
    let cjk_font = Font::with_name("Noto Sans CJK SC");

    // Load settings to get window size
    let settings = AppSettings::load().unwrap_or_default();
    let window_width = settings.gui.window_width.max(800) as f32;
    let window_height = settings.gui.window_height.max(600) as f32;

    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .default_font(cjk_font)
        .window_size((window_width, window_height))
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
