//! Main application module
//!
//! Defines the core App struct and iced integration.

use anyhow::Result;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length, Subscription, Task, Theme};

use crate::messages::Message;
use crate::state::AppState;
use crate::theme::WayvidTheme;
use crate::views::{self, View};

/// Main application struct
pub struct App {
    /// Application state
    state: AppState,
    /// Current theme
    theme: WayvidTheme,
}

impl App {
    /// Create a new application instance
    pub fn new() -> (Self, Task<Message>) {
        let (state, task) = AppState::new();

        (
            Self {
                state,
                theme: WayvidTheme::Dark,
            },
            task,
        )
    }

    /// Application title
    pub fn title(&self) -> String {
        format!("Wayvid - {}", self.state.current_view.title())
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
                        self.state.wallpapers = wallpapers;
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Failed to load library: {}", e));
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
                        self.state.status_message = Some("Wallpaper applied successfully".into());
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Failed to apply wallpaper: {}", e));
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
                        self.state.error = Some(format!("Scan failed: {}", e));
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

            // Settings
            Message::ToggleAutostart(enabled) => {
                self.state.settings.autostart = enabled;
                Task::none()
            }
            Message::ToggleMinimizeToTray(enabled) => {
                self.state.settings.minimize_to_tray = enabled;
                Task::none()
            }
            Message::SaveSettings => Task::perform(
                {
                    let settings = self.state.settings.clone();
                    async move { save_settings(&settings).await }
                },
                Message::SettingsSaved,
            ),
            Message::SettingsSaved(result) => {
                match result {
                    Ok(()) => {
                        self.state.status_message = Some("Settings saved".into());
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Failed to save settings: {}", e));
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
                self.state.daemon_connected = true;
                Task::none()
            }
            Message::DaemonDisconnected => {
                self.state.daemon_connected = false;
                Task::none()
            }

            // Thumbnail loading
            Message::ThumbnailLoaded(id, data) => {
                self.state.thumbnails.insert(id, data);
                Task::none()
            }
        }
    }

    /// Render the view
    pub fn view(&self) -> Element<Message> {
        let content: Element<Message> = match self.state.current_view {
            View::Library => views::library::view(&self.state),
            View::Folders => views::folders::view(&self.state),
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

    /// Render the sidebar navigation
    fn sidebar(&self) -> Element<Message> {
        fn nav_button(label: &'static str, view: View, current: View) -> Element<'static, Message> {
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

        let nav = column![
            text("Wayvid").size(24),
            Space::with_height(20),
            nav_button("Library", View::Library, self.state.current_view),
            nav_button("Folders", View::Folders, self.state.current_view),
            nav_button("Settings", View::Settings, self.state.current_view),
            nav_button("About", View::About, self.state.current_view),
        ]
        .spacing(5)
        .padding(15)
        .width(Length::Fixed(200.0));

        // Status indicator at bottom
        let status = if self.state.daemon_connected {
            row![
                text("●").style(|_| text::Style {
                    color: Some(iced::Color::from_rgb(0.0, 0.8, 0.0))
                }),
                text(" Daemon running")
            ]
        } else {
            row![
                text("●").style(|_| text::Style {
                    color: Some(iced::Color::from_rgb(0.8, 0.0, 0.0))
                }),
                text(" Daemon stopped")
            ]
        };

        let sidebar_content = column![
            nav,
            Space::with_height(Length::Fill),
            container(status).padding(15),
        ]
        .height(Length::Fill);

        container(sidebar_content)
            .style(container::bordered_box)
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
                text("Error").size(18),
                text(error_text),
                button("Dismiss").on_press(Message::DismissError),
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
        Subscription::none()
    }
}

/// Run the application
pub fn run() -> Result<()> {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .window_size((1200.0, 800.0))
        .run_with(App::new)?;

    Ok(())
}

// Async helper functions
async fn load_library() -> Result<Vec<wayvid_core::WallpaperItem>, String> {
    // TODO: Load from wayvid-library database
    Ok(vec![])
}

async fn apply_wallpaper(id: &str) -> Result<(), String> {
    // TODO: Send IPC command to daemon
    tracing::info!("Applying wallpaper: {}", id);
    Ok(())
}

async fn scan_folder(path: &std::path::Path) -> Result<Vec<wayvid_core::WallpaperItem>, String> {
    use wayvid_library::FolderScanner;

    let scanner = FolderScanner::new();
    scanner.scan_folder(path, true).map_err(|e| e.to_string())
}

async fn save_settings(settings: &crate::state::Settings) -> Result<(), String> {
    // TODO: Save to config file
    tracing::info!("Saving settings: {:?}", settings);
    Ok(())
}
