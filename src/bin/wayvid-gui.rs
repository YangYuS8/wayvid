//! wayvid GUI Control Panel with simplified UX (inspired by Wallpaper Engine)
//!
//! Key UX improvements:
//! - Bottom monitor selector bar (always visible)
//! - Single-click to select wallpaper, double-click to apply
//! - Unified wallpaper library (local + Workshop)
//! - Drag-and-drop support for files

use eframe::egui;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use wayvid::ctl::ipc_client::IpcClient;
use wayvid::ctl::protocol::{IpcCommand, IpcResponse};

// Initialize i18n with locales
rust_i18n::i18n!("locales", fallback = "en");

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("wayvid_gui=debug,wayvid=debug")
        .init();

    // Detect system locale and set language
    if let Some(locale) = sys_locale::get_locale() {
        let lang = if locale.starts_with("zh") {
            "zh-CN"
        } else {
            "en"
        };
        rust_i18n::set_locale(lang);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title(t!("app_title")),
        ..Default::default()
    };

    eframe::run_native(
        "wayvid",
        options,
        Box::new(|cc| {
            // Configure modern visual style
            setup_custom_style(&cc.egui_ctx);
            Ok(Box::new(WayvidApp::default()))
        }),
    )
}

/// Setup custom visual style for modern look
fn setup_custom_style(ctx: &egui::Context) {
    // Add Chinese font support
    let mut fonts = egui::FontDefinitions::default();

    // Load system Chinese font (Noto Sans CJK SC is commonly available on Linux)
    let chinese_font_paths = [
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/OTF/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/TTF/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
        "/usr/share/fonts/wenquanyi/wqy-zenhei/wqy-zenhei.ttc",
    ];

    let mut font_loaded = false;
    for font_path in &chinese_font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("chinese_font".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("chinese_font".to_owned());

            font_loaded = true;
            tracing::info!("Loaded Chinese font from: {}", font_path);
            break;
        }
    }

    if !font_loaded {
        tracing::warn!("No Chinese font found. Chinese characters may not display correctly.");
    }

    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();

    // Rounded corners for modern look
    style.visuals.window_rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(6.0);

    // Better spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);

    ctx.set_style(style);
}

struct WayvidApp {
    // IPC communication
    ipc_tx: Option<Sender<IpcCommand>>,
    ipc_rx: Option<Receiver<IpcResponse>>,

    // UI state - simplified tabs
    selected_tab: Tab,
    outputs: Vec<OutputInfo>,
    selected_output: Option<usize>, // Currently selected monitor

    // Wallpaper library (unified local + workshop)
    wallpapers: Vec<WallpaperItem>,
    selected_wallpaper: Option<usize>,
    last_click_time: Option<std::time::Instant>,
    last_clicked_wallpaper: Option<usize>,

    // Input fields
    video_path_input: String,
    url_input: String,
    library_search: String,

    // Config editing
    config_layout: String,
    config_volume: f32,
    config_mute: bool,
    config_loop: bool,
    config_hwdec: bool,

    // Render backend settings
    config_render_backend: GuiRenderBackend,
    vulkan_available: bool,
    vulkan_info: Option<String>,

    // Custom wallpaper folders
    wallpaper_folders: Vec<String>,
    new_folder_path: String, // Input field for new folder path

    // Config file editing
    config_path: String,
    config_content: String,
    config_edited: bool,
    show_config_editor: bool,

    // Language selection
    current_language: String,

    // Status
    status_message: String,
    connection_status: ConnectionStatus,
    workshop_scan_running: bool,

    // Dropped files handling
    dropped_files: Vec<egui::DroppedFile>,

    // Preview image cache (static images and GIF animations)
    preview_textures: std::collections::HashMap<String, Option<egui::TextureHandle>>,
    gif_animations: std::collections::HashMap<String, GifAnimation>,
}

/// GIF animation data for preview
struct GifAnimation {
    frames: Vec<egui::TextureHandle>,
    frame_delays: Vec<std::time::Duration>,
    current_frame: usize,
    last_update: std::time::Instant,
}

impl Clone for GifAnimation {
    fn clone(&self) -> Self {
        Self {
            frames: self.frames.clone(),
            frame_delays: self.frame_delays.clone(),
            current_frame: self.current_frame,
            last_update: self.last_update,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Library,  // Unified wallpaper library (local + workshop)
    Settings, // Settings and config
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct OutputInfo {
    name: String,
    width: u32,
    height: u32,
    active: bool,
    current_source: Option<String>,
}

/// Unified wallpaper item (can be local file, directory, URL, or Workshop item)
#[derive(Clone)]
#[allow(dead_code)]
struct WallpaperItem {
    id: String,          // Unique identifier
    name: String,        // Display name
    source_path: String, // Full path or URL
    source_type: WallpaperSource,
    wallpaper_type: WallpaperType, // Video or Scene
    preview_path: Option<String>,  // Path to preview image (gif/jpg)
    is_valid: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum WallpaperSource {
    LocalFile,
    Directory,
    Url,
    Workshop { workshop_id: u64 },
}

/// Type of wallpaper content
#[derive(Clone, Debug, PartialEq, Default)]
enum WallpaperType {
    #[default]
    Video,
    Scene,
}

/// Render backend selection for GUI
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum GuiRenderBackend {
    #[default]
    Auto,
    #[serde(rename = "opengl")]
    OpenGL,
    Vulkan,
}

impl GuiRenderBackend {
    fn display_name(&self) -> &'static str {
        match self {
            GuiRenderBackend::Auto => "Auto",
            GuiRenderBackend::OpenGL => "OpenGL",
            GuiRenderBackend::Vulkan => "Vulkan",
        }
    }
}

/// Persistent GUI configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct GuiConfig {
    /// Selected render backend
    #[serde(default)]
    render_backend: GuiRenderBackend,

    /// Custom wallpaper folder paths
    #[serde(default)]
    wallpaper_folders: Vec<String>,

    /// User's preferred language
    #[serde(default)]
    language: Option<String>,
}

impl GuiConfig {
    /// Get the GUI config file path
    fn config_path() -> std::path::PathBuf {
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            std::path::PathBuf::from(xdg_config).join("wayvid/gui.yaml")
        } else if let Ok(home) = std::env::var("HOME") {
            std::path::PathBuf::from(home).join(".config/wayvid/gui.yaml")
        } else {
            std::path::PathBuf::from("gui.yaml")
        }
    }

    /// Load config from file, or return default if file doesn't exist
    fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_yaml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse GUI config: {}", e),
                },
                Err(e) => eprintln!("Failed to read GUI config: {}", e),
            }
        }
        Self::default()
    }

    /// Save config to file
    fn save(&self) -> Result<(), String> {
        let path = Self::config_path();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&path, content).map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    #[allow(dead_code)]
    Error,
}

impl Default for WayvidApp {
    fn default() -> Self {
        // Load persistent GUI config
        let gui_config = GuiConfig::load();

        // Use saved language or detect from system
        let current_language = gui_config
            .language
            .clone()
            .unwrap_or_else(|| rust_i18n::locale().to_string());

        // Apply saved language
        rust_i18n::set_locale(&current_language);

        // Check Vulkan availability at startup
        #[cfg(feature = "backend-vulkan")]
        let (vulkan_available, vulkan_info) = {
            let info = wayvid::video::vulkan::check_vulkan_availability();
            let available = info.available;
            let info_str = if available {
                Some(format!(
                    "{} ({})",
                    info.gpu_name.unwrap_or_else(|| "Unknown GPU".to_string()),
                    info.api_version.unwrap_or_else(|| "?".to_string())
                ))
            } else {
                info.error
            };
            (available, info_str)
        };

        #[cfg(not(feature = "backend-vulkan"))]
        let (vulkan_available, vulkan_info) = (false, Some("Vulkan not compiled in".to_string()));

        Self {
            ipc_tx: None,
            ipc_rx: None,
            selected_tab: Tab::Library,
            outputs: Vec::new(),
            selected_output: None,
            wallpapers: Vec::new(),
            selected_wallpaper: None,
            last_click_time: None,
            last_clicked_wallpaper: None,
            video_path_input: String::new(),
            url_input: String::new(),
            library_search: String::new(),
            config_layout: "Fill".to_string(),
            config_volume: 0.5,
            config_mute: true,
            config_loop: true,
            config_hwdec: true,
            config_render_backend: gui_config.render_backend,
            vulkan_available,
            vulkan_info,
            wallpaper_folders: gui_config.wallpaper_folders,
            new_folder_path: String::new(),
            config_path: String::new(),
            config_content: String::new(),
            config_edited: false,
            show_config_editor: false,
            current_language,
            status_message: t!("msg_not_connected").to_string(),
            connection_status: ConnectionStatus::Disconnected,
            workshop_scan_running: false,
            dropped_files: Vec::new(),
            preview_textures: std::collections::HashMap::new(),
            gif_animations: std::collections::HashMap::new(),
        }
    }
}

impl eframe::App for WayvidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-connect if daemon is running and we're disconnected
        if self.connection_status == ConnectionStatus::Disconnected && IpcClient::is_running() {
            self.connect_ipc();
        }

        // Poll for IPC responses
        self.poll_responses();

        // Handle dropped files
        self.handle_dropped_files(ctx);

        // Request repaint for continuous updates
        ctx.request_repaint();

        // Top panel with connection status and controls
        egui::TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(16.0, 10.0)),
            )
            .show(ctx, |ui| {
                self.show_top_panel(ui);
            });

        // Bottom panel with monitor selector (like Wallpaper Engine)
        egui::TopBottomPanel::bottom("monitor_panel")
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .min_height(80.0)
            .show(ctx, |ui| {
                self.show_monitor_selector(ui);
            });

        // Left navigation panel (simplified)
        egui::SidePanel::left("side_panel")
            .default_width(160.0)
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                self.show_navigation(ui);
            });

        // Central panel - Main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| match self.selected_tab {
                Tab::Library => self.show_library_tab(ui),
                Tab::Settings => self.show_settings_tab(ui),
            });

        // Show drag-and-drop overlay when dragging files
        self.show_drop_overlay(ctx);
    }
}

impl WayvidApp {
    /// Show top panel with status and controls
    fn show_top_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(t!("panel_heading"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Daemon management buttons
                let daemon_running = IpcClient::is_running();

                if daemon_running {
                    if ui.button(t!("panel_daemon_stop")).clicked() {
                        self.stop_daemon();
                    }
                    if ui.button(t!("panel_daemon_restart")).clicked() {
                        self.restart_daemon();
                    }
                } else if ui.button(t!("panel_daemon_start")).clicked() {
                    self.start_daemon();
                }

                ui.separator();

                // Connection status indicator
                let (color, text) = match self.connection_status {
                    ConnectionStatus::Connected => {
                        (egui::Color32::from_rgb(76, 175, 80), t!("status_connected"))
                    }
                    ConnectionStatus::Connecting => (
                        egui::Color32::from_rgb(255, 193, 7),
                        t!("status_connecting"),
                    ),
                    ConnectionStatus::Disconnected => {
                        (egui::Color32::GRAY, t!("status_disconnected"))
                    }
                    ConnectionStatus::Error => {
                        (egui::Color32::from_rgb(244, 67, 54), t!("status_error"))
                    }
                };
                ui.colored_label(color, text);

                if ui.button(t!("btn_refresh")).clicked() {
                    self.refresh_outputs();
                }
            });
        });

        // Status message
        ui.horizontal(|ui| {
            ui.label(&self.status_message);
        });
    }

    /// Show bottom monitor selector panel (like Wallpaper Engine)
    fn show_monitor_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(t!("monitor_selector")).strong());
            ui.separator();

            if self.outputs.is_empty() {
                ui.label(t!("outputs_empty_hint"));
                return;
            }

            // Show all monitors horizontally with preview
            let outputs = self.outputs.clone();
            for (idx, output) in outputs.iter().enumerate() {
                let is_selected = self.selected_output == Some(idx);

                // Monitor card
                let response = egui::Frame::none()
                    .fill(if is_selected {
                        ui.style().visuals.selection.bg_fill
                    } else {
                        ui.style().visuals.widgets.noninteractive.bg_fill
                    })
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(8.0))
                    .stroke(if is_selected {
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(76, 175, 80))
                    } else {
                        egui::Stroke::NONE
                    })
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            // Monitor icon and name
                            ui.horizontal(|ui| {
                                ui.label("📺");
                                ui.label(egui::RichText::new(&output.name).strong());
                            });

                            // Resolution
                            ui.label(
                                egui::RichText::new(format!("{}×{}", output.width, output.height))
                                    .small()
                                    .color(egui::Color32::GRAY),
                            );

                            // Status
                            if output.active {
                                ui.colored_label(
                                    egui::Color32::from_rgb(76, 175, 80),
                                    t!("monitor_playing"),
                                );
                            } else {
                                ui.colored_label(egui::Color32::GRAY, t!("monitor_inactive"));
                            }
                        });
                    })
                    .response;

                // Handle click to select monitor
                if response.clicked() {
                    self.selected_output = Some(idx);
                    self.status_message =
                        t!("monitor_selected", name = output.name.clone()).to_string();
                }

                // Tooltip
                response.on_hover_text(t!("monitor_click_hint"));

                ui.add_space(8.0);
            }

            // All monitors option
            ui.separator();
            let all_selected = self.selected_output.is_none() && !self.outputs.is_empty();
            let all_response = egui::Frame::none()
                .fill(if all_selected {
                    ui.style().visuals.selection.bg_fill
                } else {
                    ui.style().visuals.widgets.noninteractive.bg_fill
                })
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(8.0))
                .stroke(if all_selected {
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(76, 175, 80))
                } else {
                    egui::Stroke::NONE
                })
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label("🖥️");
                        ui.label(egui::RichText::new(t!("monitor_all")).strong());
                        ui.label(
                            egui::RichText::new(t!("monitor_all_hint"))
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                    });
                })
                .response;

            if all_response.clicked() && !self.outputs.is_empty() {
                self.selected_output = None; // None means all monitors
                self.status_message = t!("monitor_all_selected").to_string();
            }
        });
    }

    /// Show navigation panel (simplified: just Library and Settings)
    fn show_navigation(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("nav_title"));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Simplified navigation - just two tabs
        self.nav_button(ui, Tab::Library, &t!("nav_library"));
        self.nav_button(ui, Tab::Settings, &t!("nav_settings"));

        ui.add_space(16.0);
        ui.separator();

        // Quick actions in sidebar
        ui.add_space(8.0);
        ui.label(egui::RichText::new(t!("quick_actions")).strong());

        if ui.button(t!("action_scan_workshop")).clicked() {
            self.scan_workshop();
        }

        if ui.button(t!("action_add_folder")).clicked() {
            self.status_message = t!("action_add_folder_hint").to_string();
        }
    }

    /// Custom navigation button with selection state
    fn nav_button(&mut self, ui: &mut egui::Ui, tab: Tab, label: &str) {
        let is_selected = self.selected_tab == tab;
        let button = egui::Button::new(label)
            .fill(if is_selected {
                ui.style().visuals.selection.bg_fill
            } else {
                egui::Color32::TRANSPARENT
            })
            .min_size(egui::vec2(ui.available_width(), 36.0));

        if ui.add(button).clicked() {
            self.selected_tab = tab;
        }
        ui.add_space(4.0);
    }

    /// Show unified wallpaper library (local + workshop)
    fn show_library_tab(&mut self, ui: &mut egui::Ui) {
        // Search and filter bar
        ui.horizontal(|ui| {
            ui.heading(t!("library_title"));
            ui.separator();

            // Search box
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.library_search)
                    .desired_width(200.0)
                    .hint_text(t!("library_search_hint")),
            );

            ui.separator();

            // Quick add buttons
            if ui.button(t!("library_add_file")).clicked() {
                self.status_message = t!("library_add_file_hint").to_string();
            }

            // Stats
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(t!("library_count", count = self.wallpapers.len()));
            });
        });

        ui.add_space(8.0);
        ui.separator();

        // Add URL/path input section
        self.show_quick_add_section(ui);

        ui.add_space(8.0);

        // Wallpaper grid
        if self.wallpapers.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(t!("library_empty"));
                ui.label(t!("library_empty_hint"));
                ui.add_space(16.0);

                if ui.button(t!("action_scan_workshop")).clicked() {
                    self.scan_workshop();
                }

                ui.add_space(8.0);
                ui.label(t!("library_drag_hint"));
            });
        } else {
            self.show_wallpaper_grid(ui);
        }
    }

    /// Show quick add section for path/URL input
    fn show_quick_add_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t!("quick_add_title"));
                    ui.separator();

                    // File path input
                    ui.label(t!("quick_add_path"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.video_path_input)
                            .desired_width(300.0)
                            .hint_text("~/Videos/wallpaper.mp4"),
                    );

                    if ui.button(t!("btn_apply_now")).clicked() {
                        self.apply_wallpaper_from_input();
                    }

                    ui.separator();

                    // URL input
                    ui.label("URL:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.url_input)
                            .desired_width(200.0)
                            .hint_text("https://..."),
                    );

                    if ui.button("▶").clicked() {
                        self.apply_url_wallpaper();
                    }
                });
            });
    }

    /// Show wallpaper grid with click-to-select, double-click-to-apply
    #[allow(clippy::type_complexity)]
    fn show_wallpaper_grid(&mut self, ui: &mut egui::Ui) {
        // Filter wallpapers by search - collect indices and data we need
        let search = self.library_search.to_lowercase();
        let filtered: Vec<(
            usize,
            String,
            String,
            WallpaperSource,
            WallpaperType,
            Option<String>,
            bool,
        )> = self
            .wallpapers
            .iter()
            .enumerate()
            .filter(|(_, w)| {
                search.is_empty()
                    || w.name.to_lowercase().contains(&search)
                    || w.source_path.to_lowercase().contains(&search)
            })
            .map(|(idx, w)| {
                (
                    idx,
                    w.name.clone(),
                    w.source_path.clone(),
                    w.source_type.clone(),
                    w.wallpaper_type.clone(),
                    w.preview_path.clone(),
                    w.is_valid,
                )
            })
            .collect();

        // Track actions to perform after the UI loop
        let mut wallpaper_to_apply: Option<usize> = None;
        let mut wallpaper_to_select: Option<usize> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Grid layout
            let num_columns = ((ui.available_width() - 20.0) / 280.0).max(1.0) as usize;

            egui::Grid::new("wallpaper_grid")
                .num_columns(num_columns)
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    let mut column = 0;

                    for (
                        idx,
                        name,
                        source_path,
                        source_type,
                        wallpaper_type,
                        preview_path,
                        is_valid,
                    ) in &filtered
                    {
                        let is_selected = self.selected_wallpaper == Some(*idx);

                        let response = egui::Frame::none()
                            .fill(if is_selected {
                                ui.style().visuals.selection.bg_fill.gamma_multiply(0.4)
                            } else {
                                ui.style().visuals.widgets.noninteractive.bg_fill
                            })
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(egui::Margin::same(12.0))
                            .stroke(if is_selected {
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(76, 175, 80))
                            } else {
                                egui::Stroke::NONE
                            })
                            .show(ui, |ui| {
                                ui.set_min_width(240.0);
                                ui.set_max_width(240.0);

                                ui.vertical(|ui| {
                                    // Preview image area (if available)
                                    if let Some(ref preview) = preview_path {
                                        self.show_preview_image(ui, preview);
                                    } else {
                                        // Default placeholder
                                        ui.add_sized(
                                            [224.0, 126.0],
                                            egui::Label::new(
                                                egui::RichText::new("🖼️")
                                                    .size(48.0)
                                                    .color(egui::Color32::GRAY),
                                            ),
                                        );
                                    }
                                    ui.add_space(8.0);

                                    // Icon and type badges
                                    ui.horizontal(|ui| {
                                        // Source type icon
                                        let icon = match source_type {
                                            WallpaperSource::LocalFile => "📁",
                                            WallpaperSource::Directory => "📂",
                                            WallpaperSource::Url => "🌐",
                                            WallpaperSource::Workshop { .. } => "🎮",
                                        };
                                        ui.label(icon);

                                        // Wallpaper type badge
                                        match wallpaper_type {
                                            WallpaperType::Video => {
                                                ui.colored_label(
                                                    egui::Color32::from_rgb(33, 150, 243),
                                                    "▶ Video",
                                                );
                                            }
                                            WallpaperType::Scene => {
                                                ui.colored_label(
                                                    egui::Color32::from_rgb(156, 39, 176),
                                                    "🎬 Scene",
                                                );
                                            }
                                        }
                                    });

                                    // Title
                                    ui.label(egui::RichText::new(name).strong());

                                    // Validity indicator
                                    if *is_valid {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(76, 175, 80),
                                            t!("wallpaper_valid"),
                                        );
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 193, 7),
                                            t!("wallpaper_unverified"),
                                        );
                                    }

                                    // Path (truncated)
                                    let display_path = if source_path.len() > 35 {
                                        format!("...{}", &source_path[source_path.len() - 32..])
                                    } else {
                                        source_path.clone()
                                    };
                                    ui.label(
                                        egui::RichText::new(display_path)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                    );

                                    // Apply button
                                    ui.add_space(4.0);
                                    if ui.button(t!("wallpaper_apply")).clicked() {
                                        wallpaper_to_apply = Some(*idx);
                                    }
                                });
                            })
                            .response;

                        // Handle click for selection and double-click for apply
                        if response.clicked() {
                            let now = std::time::Instant::now();
                            let is_double_click = self.last_clicked_wallpaper == Some(*idx)
                                && self
                                    .last_click_time
                                    .is_some_and(|t| now.duration_since(t).as_millis() < 400);

                            if is_double_click {
                                // Double-click: apply wallpaper
                                wallpaper_to_apply = Some(*idx);
                            } else {
                                // Single click: select
                                wallpaper_to_select = Some(*idx);
                            }

                            self.last_click_time = Some(now);
                            self.last_clicked_wallpaper = Some(*idx);
                        }

                        // Tooltip
                        response.on_hover_text(t!("wallpaper_double_click_hint"));

                        column += 1;
                        if column >= num_columns {
                            ui.end_row();
                            column = 0;
                        }
                    }
                });
        });

        // Apply actions after UI loop
        if let Some(idx) = wallpaper_to_select {
            self.selected_wallpaper = Some(idx);
        }
        if let Some(idx) = wallpaper_to_apply {
            self.apply_wallpaper(idx);
        }
    }

    /// Show preview image for a wallpaper (supports static images and GIF animations)
    fn show_preview_image(&mut self, ui: &mut egui::Ui, preview_path: &str) {
        let preview_size = egui::vec2(224.0, 126.0); // 16:9 aspect ratio
        let is_gif = preview_path.to_lowercase().ends_with(".gif");

        if is_gif {
            // Handle GIF animation
            self.show_gif_preview(ui, preview_path, preview_size);
        } else {
            // Handle static image
            self.show_static_preview(ui, preview_path, preview_size);
        }
    }

    /// Show static image preview
    fn show_static_preview(
        &mut self,
        ui: &mut egui::Ui,
        preview_path: &str,
        preview_size: egui::Vec2,
    ) {
        // Check if texture is already cached
        if let Some(texture_opt) = self.preview_textures.get(preview_path) {
            if let Some(texture) = texture_opt {
                ui.add(egui::Image::from_texture(texture).fit_to_exact_size(preview_size));
                return;
            } else {
                // Loading failed previously, show placeholder
                Self::show_placeholder(ui, preview_size);
                return;
            }
        }

        // Try to load the image
        let texture = match image::open(preview_path) {
            Ok(img) => {
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture =
                    ui.ctx()
                        .load_texture(preview_path, color_image, egui::TextureOptions::LINEAR);
                Some(texture)
            }
            Err(_) => None,
        };

        // Cache the result
        self.preview_textures
            .insert(preview_path.to_string(), texture.clone());

        // Display
        if let Some(ref tex) = texture {
            ui.add(egui::Image::from_texture(tex).fit_to_exact_size(preview_size));
        } else {
            Self::show_placeholder(ui, preview_size);
        }
    }

    /// Show GIF animation preview
    fn show_gif_preview(
        &mut self,
        ui: &mut egui::Ui,
        preview_path: &str,
        preview_size: egui::Vec2,
    ) {
        // Check if GIF is already loaded
        if let Some(anim) = self.gif_animations.get_mut(preview_path) {
            // Check if we need to advance to the next frame
            let now = std::time::Instant::now();
            if let Some(delay) = anim.frame_delays.get(anim.current_frame) {
                if now.duration_since(anim.last_update) >= *delay {
                    anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
                    anim.last_update = now;
                }
            }

            // Display current frame
            if let Some(frame) = anim.frames.get(anim.current_frame) {
                ui.add(egui::Image::from_texture(frame).fit_to_exact_size(preview_size));
                // Request repaint for animation
                if let Some(delay) = anim.frame_delays.get(anim.current_frame) {
                    ui.ctx().request_repaint_after(*delay);
                }
            } else {
                Self::show_placeholder(ui, preview_size);
            }
            return;
        }

        // Try to load GIF frames
        let anim = self.load_gif_animation(ui.ctx(), preview_path);

        if let Some(ref a) = anim {
            if let Some(frame) = a.frames.first() {
                ui.add(egui::Image::from_texture(frame).fit_to_exact_size(preview_size));
                // Request repaint for animation
                if !a.frame_delays.is_empty() {
                    ui.ctx().request_repaint_after(a.frame_delays[0]);
                }
            } else {
                Self::show_placeholder(ui, preview_size);
            }
        } else {
            Self::show_placeholder(ui, preview_size);
        }

        // Cache the animation (even if None to avoid retry)
        if let Some(a) = anim {
            self.gif_animations.insert(preview_path.to_string(), a);
        }
    }

    /// Load GIF animation frames
    fn load_gif_animation(&self, ctx: &egui::Context, path: &str) -> Option<GifAnimation> {
        use image::codecs::gif::GifDecoder;
        use image::AnimationDecoder;
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);
        let decoder = GifDecoder::new(reader).ok()?;
        let frames_iter = decoder.into_frames();

        let mut frames = Vec::new();
        let mut frame_delays = Vec::new();

        for frame in frames_iter.flatten() {
            let delay = frame.delay();
            // Convert delay to Duration (delay is in units of 1/100 second)
            let (numer, denom) = delay.numer_denom_ms();
            let delay_ms = (numer as f64) / (denom as f64);
            // Minimum delay of 20ms to prevent too fast animation
            let delay_duration = std::time::Duration::from_millis(delay_ms.max(20.0) as u64);

            let img = frame.into_buffer();
            let size = [img.width() as usize, img.height() as usize];
            let pixels = img.into_raw();

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            let texture_name = format!("{}_frame_{}", path, frames.len());
            let texture =
                ctx.load_texture(&texture_name, color_image, egui::TextureOptions::LINEAR);

            frames.push(texture);
            frame_delays.push(delay_duration);
        }

        if frames.is_empty() {
            return None;
        }

        Some(GifAnimation {
            frames,
            frame_delays,
            current_frame: 0,
            last_update: std::time::Instant::now(),
        })
    }

    /// Show placeholder for missing preview
    fn show_placeholder(ui: &mut egui::Ui, preview_size: egui::Vec2) {
        ui.add_sized(
            preview_size,
            egui::Label::new(
                egui::RichText::new("🖼️")
                    .size(48.0)
                    .color(egui::Color32::GRAY),
            ),
        );
    }

    /// Apply wallpaper to selected monitor(s)
    fn apply_wallpaper(&mut self, wallpaper_idx: usize) {
        if let Some(wallpaper) = self.wallpapers.get(wallpaper_idx) {
            let source_path = wallpaper.source_path.clone();
            let wallpaper_name = wallpaper.name.clone();
            let wallpaper_type = wallpaper.wallpaper_type.clone();

            // Determine target output(s)
            let output_name = if let Some(idx) = self.selected_output {
                self.outputs.get(idx).map(|o| o.name.clone())
            } else {
                None // Apply to all
            };

            // Use appropriate source type based on wallpaper type
            match wallpaper_type {
                WallpaperType::Scene => {
                    // For scene wallpapers, use SwitchSource with WeScene type
                    use wayvid::core::types::VideoSource;
                    self.send_command(IpcCommand::SwitchSource {
                        output: output_name.clone(),
                        source: VideoSource::WeScene {
                            path: source_path.clone(),
                        },
                    });
                }
                WallpaperType::Video => {
                    // For video wallpapers, use SetSource with path string
                    self.send_command(IpcCommand::SetSource {
                        output: output_name.clone(),
                        source: source_path,
                    });
                }
            }

            let target = output_name.unwrap_or_else(|| t!("monitor_all").to_string());
            self.status_message =
                t!("wallpaper_applied", name = wallpaper_name, target = target).to_string();
        }
    }

    /// Apply wallpaper from path input field
    fn apply_wallpaper_from_input(&mut self) {
        if self.video_path_input.is_empty() {
            self.status_message = t!("error_empty_path").to_string();
            return;
        }

        let source_path = self.video_path_input.clone();

        let output_name = if let Some(idx) = self.selected_output {
            self.outputs.get(idx).map(|o| o.name.clone())
        } else {
            None
        };

        self.send_command(IpcCommand::SetSource {
            output: output_name.clone(),
            source: source_path.clone(),
        });

        let target = output_name.unwrap_or_else(|| t!("monitor_all").to_string());
        self.status_message = t!(
            "wallpaper_applied",
            name = source_path.clone(),
            target = target
        )
        .to_string();

        // Add to library
        self.add_to_library(&source_path);
    }

    /// Apply URL wallpaper
    fn apply_url_wallpaper(&mut self) {
        if self.url_input.is_empty() {
            self.status_message = t!("error_empty_url").to_string();
            return;
        }

        let url = self.url_input.clone();

        let output_name = if let Some(idx) = self.selected_output {
            self.outputs.get(idx).map(|o| o.name.clone())
        } else {
            None
        };

        self.send_command(IpcCommand::SetSource {
            output: output_name.clone(),
            source: url.clone(),
        });

        let target = output_name.unwrap_or_else(|| t!("monitor_all").to_string());
        self.status_message = t!("wallpaper_applied", name = url, target = target).to_string();
    }

    /// Add a path to the library
    fn add_to_library(&mut self, path: &str) {
        // Check if already in library
        if self.wallpapers.iter().any(|w| w.source_path == path) {
            return;
        }

        let source_type = if path.starts_with("http") {
            WallpaperSource::Url
        } else if std::path::Path::new(path).is_dir() {
            WallpaperSource::Directory
        } else {
            WallpaperSource::LocalFile
        };

        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        self.wallpapers.push(WallpaperItem {
            id: format!("local-{}", self.wallpapers.len()),
            name,
            source_path: path.to_string(),
            source_type,
            wallpaper_type: WallpaperType::Video,
            preview_path: None,
            is_valid: true,
        });
    }

    /// Handle dropped files
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        // Collect dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();
            }
        });

        // Process dropped files
        for file in std::mem::take(&mut self.dropped_files) {
            if let Some(path) = file.path {
                let path_str = path.to_string_lossy().to_string();
                self.video_path_input = path_str.clone();
                self.add_to_library(&path_str);

                // Auto-apply if monitor is selected
                if self.selected_output.is_some() || !self.outputs.is_empty() {
                    self.apply_wallpaper_from_input();
                }
            }
        }
    }

    /// Show drop overlay when dragging files
    fn show_drop_overlay(&self, ctx: &egui::Context) {
        let is_dragging = ctx.input(|i| !i.raw.hovered_files.is_empty());

        if is_dragging {
            egui::Area::new(egui::Id::new("drop_overlay"))
                .fixed_pos(egui::pos2(0.0, 0.0))
                .show(ctx, |ui| {
                    let screen = ui.ctx().screen_rect();
                    ui.painter().rect_filled(
                        screen,
                        egui::Rounding::ZERO,
                        egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
                    );

                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(screen), |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(screen.height() / 2.0 - 50.0);
                            ui.label(
                                egui::RichText::new(t!("drop_file_here"))
                                    .size(32.0)
                                    .color(egui::Color32::WHITE),
                            );
                            ui.label(
                                egui::RichText::new(t!("drop_file_hint"))
                                    .size(16.0)
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        });
                    });
                });
        }
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("settings_title"));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Language settings
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_language_title"));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(t!("settings_language_select"));
                        let old_language = self.current_language.clone();
                        egui::ComboBox::from_id_salt("language_select")
                            .selected_text(&self.current_language)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_value(
                                        &mut self.current_language,
                                        "en".to_string(),
                                        "English",
                                    )
                                    .clicked()
                                {
                                    rust_i18n::set_locale("en");
                                }
                                if ui
                                    .selectable_value(
                                        &mut self.current_language,
                                        "zh-CN".to_string(),
                                        "简体中文",
                                    )
                                    .clicked()
                                {
                                    rust_i18n::set_locale("zh-CN");
                                }
                            });
                        // Save config when language changes
                        if self.current_language != old_language {
                            self.save_gui_config();
                        }
                    });
                });

            ui.add_space(12.0);

            // Video Configuration
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_video_title"));
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label(t!("settings_video_layout"));
                        egui::ComboBox::from_id_salt("layout_mode")
                            .selected_text(&self.config_layout)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.config_layout,
                                    "Fill".to_string(),
                                    t!("settings_video_layout_fill"),
                                );
                                ui.selectable_value(
                                    &mut self.config_layout,
                                    "Contain".to_string(),
                                    t!("settings_video_layout_contain"),
                                );
                                ui.selectable_value(
                                    &mut self.config_layout,
                                    "Stretch".to_string(),
                                    t!("settings_video_layout_stretch"),
                                );
                                ui.selectable_value(
                                    &mut self.config_layout,
                                    "Cover".to_string(),
                                    t!("settings_video_layout_cover"),
                                );
                                ui.selectable_value(
                                    &mut self.config_layout,
                                    "Centre".to_string(),
                                    t!("settings_video_layout_centre"),
                                );
                            });
                    });

                    ui.add_space(8.0);
                    ui.checkbox(&mut self.config_loop, t!("settings_video_loop"));
                    ui.checkbox(&mut self.config_hwdec, t!("settings_video_hwdec"));

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.config_mute, t!("settings_video_mute"));
                        if !self.config_mute {
                            ui.label(t!("settings_video_volume"));
                            let volume_pct = (self.config_volume * 100.0) as i32;
                            ui.add(
                                egui::Slider::new(&mut self.config_volume, 0.0..=1.0)
                                    .text(format!("{}%", volume_pct)),
                            );
                        }
                    });
                });

            ui.add_space(12.0);

            // Render Backend Settings
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_renderer_title"));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(t!("settings_renderer_backend"));
                        let old_backend = self.config_render_backend.clone();
                        egui::ComboBox::from_id_salt("render_backend")
                            .selected_text(self.config_render_backend.display_name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.config_render_backend,
                                    GuiRenderBackend::Auto,
                                    "Auto",
                                );
                                ui.selectable_value(
                                    &mut self.config_render_backend,
                                    GuiRenderBackend::OpenGL,
                                    "OpenGL",
                                );
                                let vulkan_label = if self.vulkan_available {
                                    "Vulkan"
                                } else {
                                    "Vulkan (unavailable)"
                                };
                                ui.add_enabled(
                                    self.vulkan_available,
                                    egui::SelectableLabel::new(
                                        self.config_render_backend == GuiRenderBackend::Vulkan,
                                        vulkan_label,
                                    ),
                                )
                                .clicked()
                                .then(|| {
                                    if self.vulkan_available {
                                        self.config_render_backend = GuiRenderBackend::Vulkan;
                                    }
                                });
                            });
                        // Save config when backend changes
                        if self.config_render_backend != old_backend {
                            self.save_gui_config();
                        }
                    });

                    // Vulkan status info
                    if let Some(ref info) = self.vulkan_info {
                        ui.add_space(4.0);
                        if self.vulkan_available {
                            ui.colored_label(
                                egui::Color32::from_rgb(76, 175, 80),
                                format!("✓ Vulkan: {}", info),
                            );
                        } else {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 193, 7),
                                format!("⚠ {}", info),
                            );
                        }
                    }

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(t!("settings_renderer_restart_hint"))
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                });

            ui.add_space(12.0);

            // Wallpaper Folders Management
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_folders_title"));
                    ui.label(t!("settings_folders_hint"));
                    ui.add_space(8.0);

                    // List existing folders
                    let mut folder_to_remove: Option<usize> = None;
                    for (idx, folder) in self.wallpaper_folders.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label("📂");
                            ui.label(folder);
                            if ui.small_button("✕").clicked() {
                                folder_to_remove = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = folder_to_remove {
                        let removed_folder = self.wallpaper_folders.remove(idx);
                        // Remove wallpapers from this folder
                        self.wallpapers
                            .retain(|w| !w.source_path.starts_with(&removed_folder));
                        self.status_message = t!("msg_folder_removed").to_string();
                        // Save config after folder removal
                        self.save_gui_config();
                    }

                    ui.add_space(8.0);

                    // Folder path input and add button
                    ui.horizontal(|ui| {
                        ui.label(t!("settings_folders_path"));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_folder_path)
                                .desired_width(300.0)
                                .hint_text("~/Videos/Wallpapers"),
                        );
                        if ui.button(t!("settings_folders_add")).clicked()
                            && !self.new_folder_path.is_empty()
                        {
                            let expanded_path =
                                shellexpand::tilde(&self.new_folder_path).to_string();
                            if std::path::Path::new(&expanded_path).is_dir() {
                                if !self.wallpaper_folders.contains(&expanded_path) {
                                    self.wallpaper_folders.push(expanded_path.clone());
                                    self.scan_folder(&expanded_path);
                                    // Save config after folder added
                                    self.save_gui_config();
                                }
                                self.new_folder_path.clear();
                            } else {
                                self.status_message = t!("msg_folder_not_found").to_string();
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button(t!("settings_folders_scan")).clicked() {
                            self.scan_all_folders();
                        }
                    });
                });

            ui.add_space(12.0);

            // Config File Editor
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_config_title"));
                    ui.label(t!("settings_config_hint"));
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button(t!("settings_config_load")).clicked() {
                            self.load_config_file();
                            self.show_config_editor = true;
                        }

                        if self.show_config_editor {
                            if ui.button(t!("settings_config_save")).clicked() {
                                self.save_config_file();
                            }
                            if ui.button(t!("settings_config_close")).clicked() {
                                self.show_config_editor = false;
                            }
                        }
                    });

                    if self.show_config_editor {
                        ui.add_space(8.0);
                        ui.separator();

                        let text_edit = egui::TextEdit::multiline(&mut self.config_content)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(15);

                        if ui.add(text_edit).changed() {
                            self.config_edited = true;
                        }
                    }
                });

            ui.add_space(12.0);

            // About
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_about_title"));
                    ui.label(format!("wayvid v{}", env!("CARGO_PKG_VERSION")));
                    ui.label(t!("settings_about_description"));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.hyperlink_to(
                            t!("settings_about_github"),
                            "https://github.com/YangYuS8/wayvid",
                        );
                        ui.hyperlink_to(
                            t!("settings_about_docs"),
                            "https://www.yangyus8.top/wayvid/",
                        );
                    });
                });
        });
    }

    // ============ Daemon management ============

    fn start_daemon(&mut self) {
        self.status_message = t!("msg_starting_daemon").to_string();
        match std::process::Command::new("systemctl")
            .args(["--user", "start", "wayvid.service"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    self.status_message = t!("msg_daemon_started").to_string();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    self.connect_ipc();
                } else {
                    self.status_message = t!("msg_daemon_start_failed").to_string();
                }
            }
            Err(e) => {
                self.status_message =
                    t!("msg_daemon_start_error", error = e.to_string()).to_string();
            }
        }
    }

    fn stop_daemon(&mut self) {
        self.status_message = t!("msg_stopping_daemon").to_string();
        match std::process::Command::new("systemctl")
            .args(["--user", "stop", "wayvid.service"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    self.status_message = t!("msg_daemon_stopped").to_string();
                    self.connection_status = ConnectionStatus::Disconnected;
                    self.ipc_tx = None;
                    self.ipc_rx = None;
                } else {
                    self.status_message = t!("msg_daemon_stop_failed").to_string();
                }
            }
            Err(e) => {
                self.status_message =
                    t!("msg_daemon_stop_error", error = e.to_string()).to_string();
            }
        }
    }

    fn restart_daemon(&mut self) {
        self.status_message = t!("msg_restarting_daemon").to_string();
        match std::process::Command::new("systemctl")
            .args(["--user", "restart", "wayvid.service"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    self.status_message = t!("msg_daemon_restarted").to_string();
                    self.connection_status = ConnectionStatus::Disconnected;
                    self.ipc_tx = None;
                    self.ipc_rx = None;
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    self.connect_ipc();
                } else {
                    self.status_message = t!("msg_daemon_restart_failed").to_string();
                }
            }
            Err(e) => {
                self.status_message =
                    t!("msg_daemon_restart_error", error = e.to_string()).to_string();
            }
        }
    }

    // ============ GUI Config Persistence ============

    /// Save GUI settings (render backend, folders, language) to gui.yaml
    fn save_gui_config(&self) {
        let config = GuiConfig {
            render_backend: self.config_render_backend.clone(),
            wallpaper_folders: self.wallpaper_folders.clone(),
            language: Some(self.current_language.clone()),
        };

        if let Err(e) = config.save() {
            eprintln!("Failed to save GUI config: {}", e);
        }
    }

    // ============ Config file handling ============

    fn load_config_file(&mut self) {
        use std::path::PathBuf;

        let config_path = if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config).join("wayvid/config.yaml")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config/wayvid/config.yaml")
        } else {
            self.status_message = t!("msg_config_path_error").to_string();
            return;
        };

        self.config_path = config_path.to_string_lossy().to_string();

        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                self.config_content = content;
                self.config_edited = false;
                self.status_message =
                    t!("msg_config_loaded", path = self.config_path.clone()).to_string();
            }
            Err(e) => {
                self.status_message =
                    t!("msg_config_load_failed", error = e.to_string()).to_string();
                self.config_content = include_str!("../../configs/config.example.yaml").to_string();
                self.config_edited = true;
            }
        }
    }

    fn save_config_file(&mut self) {
        use std::io::Write;
        use std::path::Path;

        let path = Path::new(&self.config_path);

        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.status_message =
                    t!("msg_config_dir_failed", error = e.to_string()).to_string();
                return;
            }
        }

        match std::fs::File::create(path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(self.config_content.as_bytes()) {
                    self.status_message =
                        t!("msg_config_save_failed", error = e.to_string()).to_string();
                } else {
                    self.config_edited = false;
                    self.status_message =
                        t!("msg_config_saved", path = self.config_path.clone()).to_string();

                    if self.connection_status == ConnectionStatus::Connected {
                        self.send_command(IpcCommand::ReloadConfig);
                        self.status_message += &t!("msg_config_reloaded");
                    }
                }
            }
            Err(e) => {
                self.status_message =
                    t!("msg_config_open_failed", error = e.to_string()).to_string();
            }
        }
    }

    // ============ IPC Communication ============

    fn connect_ipc(&mut self) {
        self.connection_status = ConnectionStatus::Connecting;
        self.status_message = t!("msg_connecting").to_string();

        if !IpcClient::is_running() {
            self.connection_status = ConnectionStatus::Error;
            self.status_message = t!("msg_daemon_not_running").to_string();
            return;
        }

        let (cmd_tx, cmd_rx): (Sender<IpcCommand>, Receiver<IpcCommand>) = channel();
        let (resp_tx, resp_rx): (Sender<IpcResponse>, Receiver<IpcResponse>) = channel();

        thread::spawn(move || {
            if let Err(e) = Self::ipc_thread(cmd_rx, resp_tx) {
                eprintln!("IPC thread error: {}", e);
            }
        });

        self.ipc_tx = Some(cmd_tx);
        self.ipc_rx = Some(resp_rx);
        self.connection_status = ConnectionStatus::Connected;
        self.status_message = t!("msg_connected").to_string();

        self.send_command(IpcCommand::GetStatus);
    }

    fn ipc_thread(
        cmd_rx: Receiver<IpcCommand>,
        resp_tx: Sender<IpcResponse>,
    ) -> anyhow::Result<()> {
        for command in cmd_rx {
            match IpcClient::connect() {
                Ok(mut client) => match client.send_command(&command) {
                    Ok(response) => {
                        if resp_tx.send(response).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = resp_tx.send(IpcResponse::Error {
                            message: e.to_string(),
                        });
                    }
                },
                Err(e) => {
                    let _ = resp_tx.send(IpcResponse::Error {
                        message: format!("Connection failed: {}", e),
                    });
                }
            }
        }
        Ok(())
    }

    fn send_command(&mut self, command: IpcCommand) {
        if let Some(ref tx) = self.ipc_tx {
            if let Err(e) = tx.send(command) {
                self.status_message = t!("msg_error", message = e.to_string()).to_string();
                self.connection_status = ConnectionStatus::Error;
            }
        }
    }

    fn poll_responses(&mut self) {
        let responses: Vec<IpcResponse> = if let Some(ref rx) = self.ipc_rx {
            let mut resps = Vec::new();
            while let Ok(response) = rx.try_recv() {
                resps.push(response);
            }
            resps
        } else {
            Vec::new()
        };

        for response in responses {
            self.handle_response(response);
        }
    }

    fn handle_response(&mut self, response: IpcResponse) {
        match response {
            IpcResponse::Success { data } => {
                if let Some(value) = data {
                    if let Ok(status) =
                        serde_json::from_value::<wayvid::ctl::protocol::DaemonStatus>(value)
                    {
                        self.outputs = status
                            .outputs
                            .into_iter()
                            .map(|o| OutputInfo {
                                name: o.name,
                                width: o.width as u32,
                                height: o.height as u32,
                                active: o.playing && !o.paused,
                                current_source: Some(o.source),
                            })
                            .collect();

                        // Auto-select first output if none selected
                        if self.selected_output.is_none() && !self.outputs.is_empty() {
                            self.selected_output = Some(0);
                        }

                        self.status_message =
                            t!("msg_status_updated", count = self.outputs.len()).to_string();
                    } else {
                        self.status_message = t!("msg_command_success").to_string();
                    }
                } else {
                    self.status_message = t!("msg_command_success").to_string();
                }
            }
            IpcResponse::Error { message } => {
                self.status_message = t!("msg_error", message = message).to_string();
                self.connection_status = ConnectionStatus::Error;
            }
        }
    }

    fn refresh_outputs(&mut self) {
        if self.connection_status == ConnectionStatus::Connected {
            self.send_command(IpcCommand::GetStatus);
            self.status_message = t!("msg_refreshing").to_string();
        } else {
            self.status_message = t!("msg_not_connected_hint").to_string();
        }
    }

    // ============ Wallpaper Folder Scanning ============

    /// Scan a folder for video files and add them to the library
    fn scan_folder(&mut self, folder_path: &str) {
        use std::path::Path;

        let path = Path::new(folder_path);
        if !path.exists() || !path.is_dir() {
            self.status_message = t!("msg_folder_not_found").to_string();
            return;
        }

        let video_extensions = ["mp4", "mkv", "webm", "mov", "avi", "m4v", "wmv"];
        let mut count = 0;

        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(ext) = entry_path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if video_extensions.contains(&ext_str.as_str()) {
                            let path_str = entry_path.to_string_lossy().to_string();

                            // Skip if already in library
                            if self.wallpapers.iter().any(|w| w.source_path == path_str) {
                                continue;
                            }

                            let name = entry_path
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| "Unknown".to_string());

                            self.wallpapers.push(WallpaperItem {
                                id: format!("local-{}", path_str.replace('/', "-")),
                                name,
                                source_path: path_str,
                                source_type: WallpaperSource::LocalFile,
                                wallpaper_type: WallpaperType::Video,
                                preview_path: None,
                                is_valid: true,
                            });
                            count += 1;
                        }
                    }
                } else if entry_path.is_dir() {
                    // Recursively scan subdirectories
                    self.scan_folder(&entry_path.to_string_lossy());
                }
            }
        }

        if count > 0 {
            self.status_message = t!("msg_folder_scanned", count = count).to_string();
        }
    }

    /// Scan all configured wallpaper folders
    fn scan_all_folders(&mut self) {
        let folders: Vec<String> = self.wallpaper_folders.clone();
        for folder in folders {
            self.scan_folder(&folder);
        }
        self.status_message =
            t!("msg_folders_scanned", count = self.wallpaper_folders.len()).to_string();
    }

    // ============ Workshop Integration ============

    fn scan_workshop(&mut self) {
        use wayvid::we::steam::SteamLibrary;
        use wayvid::we::workshop::{WallpaperEngineType, WorkshopScanner, WALLPAPER_ENGINE_APP_ID};

        self.workshop_scan_running = true;
        self.status_message = t!("msg_scanning_workshop").to_string();

        match SteamLibrary::discover() {
            Ok(library) => match library.find_workshop_items(WALLPAPER_ENGINE_APP_ID) {
                Ok(paths) => match WorkshopScanner::scan(&paths) {
                    Ok(scanner) => {
                        let mut video_count = 0;
                        let mut scene_count = 0;

                        for item in scanner.items() {
                            if item.is_valid() {
                                // Determine wallpaper type
                                let (source_path, wallpaper_type) = match item.wallpaper_type() {
                                    WallpaperEngineType::Video => {
                                        if let Some(video_path) = item.video_path() {
                                            video_count += 1;
                                            (
                                                video_path.to_string_lossy().to_string(),
                                                WallpaperType::Video,
                                            )
                                        } else {
                                            continue;
                                        }
                                    }
                                    WallpaperEngineType::Scene => {
                                        // For scene wallpapers, use the project directory
                                        scene_count += 1;
                                        (
                                            item.path.to_string_lossy().to_string(),
                                            WallpaperType::Scene,
                                        )
                                    }
                                    _ => continue, // Skip web and application types for now
                                };

                                // Skip if already in library
                                if self.wallpapers.iter().any(|w| w.source_path == source_path) {
                                    continue;
                                }

                                // Get preview image path
                                let preview_path =
                                    item.preview_path().map(|p| p.to_string_lossy().to_string());

                                self.wallpapers.push(WallpaperItem {
                                    id: format!("workshop-{}", item.id),
                                    name: item.title(),
                                    source_path,
                                    source_type: WallpaperSource::Workshop {
                                        workshop_id: item.id,
                                    },
                                    wallpaper_type,
                                    preview_path,
                                    is_valid: true,
                                });
                            }
                        }

                        self.status_message = t!(
                            "msg_workshop_found_detailed",
                            total = scanner.items().len(),
                            videos = video_count,
                            scenes = scene_count
                        )
                        .to_string();
                    }
                    Err(e) => {
                        self.status_message =
                            t!("msg_workshop_error", error = e.to_string()).to_string();
                    }
                },
                Err(e) => {
                    self.status_message =
                        t!("msg_workshop_scan_error", error = e.to_string()).to_string();
                }
            },
            Err(e) => {
                self.status_message = t!("msg_steam_not_found", error = e.to_string()).to_string();
            }
        }

        self.workshop_scan_running = false;
    }
}
