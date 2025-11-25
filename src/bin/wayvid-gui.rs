//! wayvid GUI Control Panel with i18n support

use eframe::egui;
use rust_i18n::t;
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
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0])
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
    // Try multiple common Chinese fonts
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

            // Add Chinese font as fallback for proportional text
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("chinese_font".to_owned());

            // Add Chinese font as fallback for monospace text
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

    // UI state
    selected_tab: Tab,
    outputs: Vec<OutputInfo>,
    video_sources: Vec<VideoSource>,
    workshop_items: Vec<WorkshopItemInfo>,
    selected_output: Option<usize>,
    selected_workshop: Option<usize>,

    // Input fields
    video_path_input: String,
    url_input: String,
    workshop_search: String,

    // Config editing
    config_layout: String,
    config_volume: f32,
    config_mute: bool,
    config_loop: bool,
    config_hwdec: bool,

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
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Outputs,
    Sources,
    Workshop,
    Settings,
}

#[derive(Debug, Clone)]
struct OutputInfo {
    name: String,
    width: u32,
    height: u32,
    active: bool,
}

#[allow(dead_code)] // Reserved for future video library/browser feature
#[derive(Clone)]
struct VideoSource {
    path: String,
    name: String,
    thumbnail: Option<egui::TextureHandle>,
}

#[derive(Clone)]
struct WorkshopItemInfo {
    id: u64,
    title: String,
    #[allow(dead_code)] // Reserved for future file browser feature
    path: String,
    video_path: Option<String>,
    is_valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    #[allow(dead_code)] // Will be used for IPC error handling
    Error,
}

impl Default for WayvidApp {
    fn default() -> Self {
        let current_language = rust_i18n::locale().to_string();
        Self {
            ipc_tx: None,
            ipc_rx: None,
            selected_tab: Tab::Outputs,
            outputs: Vec::new(),
            video_sources: Vec::new(),
            workshop_items: Vec::new(),
            selected_output: None,
            selected_workshop: None,
            video_path_input: String::new(),
            url_input: String::new(),
            workshop_search: String::new(),
            config_layout: "Fill".to_string(),
            config_volume: 0.5,
            config_mute: true,
            config_loop: true,
            config_hwdec: true,
            config_path: String::new(),
            config_content: String::new(),
            config_edited: false,
            show_config_editor: false,
            current_language,
            status_message: t!("msg_not_connected").to_string(),
            connection_status: ConnectionStatus::Disconnected,
            workshop_scan_running: false,
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

        // Request repaint for continuous updates
        ctx.request_repaint();

        // Top panel with modern styling
        egui::TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(t!("panel_heading"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Daemon management buttons
                        let daemon_running = IpcClient::is_running();

                        if daemon_running {
                            if ui
                                .button(t!("panel_daemon_stop"))
                                .on_hover_text(t!("panel_daemon_stop_tooltip"))
                                .clicked()
                            {
                                self.stop_daemon();
                            }
                            if ui
                                .button(t!("panel_daemon_restart"))
                                .on_hover_text(t!("panel_daemon_restart_tooltip"))
                                .clicked()
                            {
                                self.restart_daemon();
                            }
                        } else if ui
                            .button(t!("panel_daemon_start"))
                            .on_hover_text(t!("panel_daemon_start_tooltip"))
                            .clicked()
                        {
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

                        if self.connection_status == ConnectionStatus::Disconnected
                            && ui.button(t!("btn_connect")).clicked()
                        {
                            self.connect_ipc();
                        }
                    });
                });

                // Show daemon status notice if not running
                if !IpcClient::is_running()
                    && self.connection_status == ConnectionStatus::Disconnected
                {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("⚠");
                        ui.colored_label(egui::Color32::from_rgb(255, 193, 7), t!("daemon_notice"));
                        ui.label(t!("daemon_notice_hint"));
                        ui.code("systemctl --user start wayvid.service");
                    });
                }
            });

        // Bottom status bar
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.status_message);
                });
            });

        // Left navigation panel with modern styling
        egui::SidePanel::left("side_panel")
            .default_width(180.0)
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                ui.heading(t!("nav_title"));
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Modern navigation buttons
                self.nav_button(ui, Tab::Outputs, &t!("nav_outputs"));
                self.nav_button(ui, Tab::Sources, &t!("nav_sources"));
                self.nav_button(ui, Tab::Workshop, &t!("nav_workshop"));
                self.nav_button(ui, Tab::Settings, &t!("nav_settings"));
            });

        // Central panel - Main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| match self.selected_tab {
                Tab::Outputs => self.show_outputs_tab(ui),
                Tab::Sources => self.show_sources_tab(ui),
                Tab::Workshop => self.show_workshop_tab(ui),
                Tab::Settings => self.show_settings_tab(ui),
            });
    }
}

impl WayvidApp {
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

    /// Start wayvid daemon via systemd
    fn start_daemon(&mut self) {
        self.status_message = t!("msg_starting_daemon").to_string();

        match std::process::Command::new("systemctl")
            .args(["--user", "start", "wayvid.service"])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    self.status_message = t!("msg_daemon_started").to_string();
                    // Wait a moment for daemon to initialize
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

    /// Stop wayvid daemon via systemd
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

    /// Restart wayvid daemon via systemd
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
                    // Wait for daemon to restart
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

    /// Load config file from default location
    fn load_config_file(&mut self) {
        use std::path::PathBuf;

        // Get default config path
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
                // Provide a default template
                self.config_content = include_str!("../../configs/config.example.yaml").to_string();
                self.config_edited = true;
            }
        }
    }

    /// Save config file
    fn save_config_file(&mut self) {
        use std::io::Write;
        use std::path::Path;

        let path = Path::new(&self.config_path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.status_message =
                    t!("msg_config_dir_failed", error = e.to_string()).to_string();
                return;
            }
        }

        // Write config file
        match std::fs::File::create(path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(self.config_content.as_bytes()) {
                    self.status_message =
                        t!("msg_config_save_failed", error = e.to_string()).to_string();
                } else {
                    self.config_edited = false;
                    self.status_message =
                        t!("msg_config_saved", path = self.config_path.clone()).to_string();

                    // Trigger config reload if daemon is connected
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

    /// Validate YAML syntax
    fn validate_config(&self) -> Result<(), String> {
        match serde_yaml::from_str::<serde_yaml::Value>(&self.config_content) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("YAML syntax error: {}", e)),
        }
    }

    fn connect_ipc(&mut self) {
        self.connection_status = ConnectionStatus::Connecting;
        self.status_message = t!("msg_connecting").to_string();

        // Check if wayvid is running
        if !IpcClient::is_running() {
            self.connection_status = ConnectionStatus::Error;
            self.status_message = t!("msg_daemon_not_running").to_string();
            return;
        }

        // Create channels for async communication
        let (cmd_tx, cmd_rx): (Sender<IpcCommand>, Receiver<IpcCommand>) = channel();
        let (resp_tx, resp_rx): (Sender<IpcResponse>, Receiver<IpcResponse>) = channel();

        // Spawn IPC thread
        thread::spawn(move || {
            if let Err(e) = Self::ipc_thread(cmd_rx, resp_tx) {
                eprintln!("IPC thread error: {}", e);
            }
        });

        self.ipc_tx = Some(cmd_tx);
        self.ipc_rx = Some(resp_rx);
        self.connection_status = ConnectionStatus::Connected;
        self.status_message = t!("msg_connected").to_string();

        // Request initial status
        self.send_command(IpcCommand::GetStatus);
    }

    /// IPC communication thread
    fn ipc_thread(
        cmd_rx: Receiver<IpcCommand>,
        resp_tx: Sender<IpcResponse>,
    ) -> anyhow::Result<()> {
        let mut client = IpcClient::connect()?;

        for command in cmd_rx {
            match client.send_command(&command) {
                Ok(response) => {
                    if resp_tx.send(response).is_err() {
                        break; // GUI closed
                    }
                }
                Err(e) => {
                    eprintln!("IPC command error: {}", e);
                    // Send error response
                    let _ = resp_tx.send(IpcResponse::Error {
                        message: e.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Send IPC command
    fn send_command(&mut self, command: IpcCommand) {
        if let Some(ref tx) = self.ipc_tx {
            if let Err(e) = tx.send(command) {
                self.status_message = t!("msg_error", message = e.to_string()).to_string();
                self.connection_status = ConnectionStatus::Error;
            }
        }
    }

    /// Poll for IPC responses
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

    /// Handle IPC response
    fn handle_response(&mut self, response: IpcResponse) {
        match response {
            IpcResponse::Success { data } => {
                if let Some(value) = data {
                    // Try to parse as daemon status
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
                            })
                            .collect();
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

    fn show_outputs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("outputs_title"));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        if self.outputs.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(t!("outputs_empty"));
                ui.label(t!("outputs_empty_hint"));
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let outputs = self.outputs.clone(); // Clone to avoid borrow conflicts
            for (idx, output) in outputs.iter().enumerate() {
                let is_selected = self.selected_output == Some(idx);

                egui::Frame::none()
                    .fill(if is_selected {
                        ui.style().visuals.selection.bg_fill.gamma_multiply(0.3)
                    } else {
                        ui.style().visuals.widgets.noninteractive.bg_fill
                    })
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Selection checkbox
                            let mut selected = is_selected;
                            if ui.checkbox(&mut selected, "").changed() {
                                self.selected_output = if selected { Some(idx) } else { None };
                            }

                            // Output info
                            ui.vertical(|ui| {
                                ui.heading(&output.name);
                                ui.label(format!("{}×{}", output.width, output.height));
                                if output.active {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(76, 175, 80),
                                        t!("outputs_active"),
                                    );
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, t!("outputs_inactive"));
                                }
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if output.active {
                                        if ui.button(t!("outputs_pause")).clicked() {
                                            self.send_command(IpcCommand::Pause {
                                                output: Some(output.name.clone()),
                                            });
                                        }
                                    } else if ui.button(t!("outputs_resume")).clicked() {
                                        self.send_command(IpcCommand::Resume {
                                            output: Some(output.name.clone()),
                                        });
                                    }

                                    if ui.button(t!("outputs_configure")).clicked() {
                                        self.status_message =
                                            t!("msg_configuring", output = output.name.clone())
                                                .to_string();
                                    }
                                },
                            );
                        });
                    });

                ui.add_space(8.0);
            }
        });
    }

    fn show_sources_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("sources_title"));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Local file section
        egui::Frame::none()
            .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.heading(t!("sources_local_title"));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(t!("sources_local_path"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.video_path_input)
                            .desired_width(ui.available_width() - 100.0),
                    );
                    if ui.button(t!("btn_browse")).clicked() {
                        // Native file dialog support would require rfd crate
                        self.status_message = t!("sources_local_hint").to_string();
                    }
                });
                ui.add_space(4.0);
                if ui.button(t!("btn_apply")).clicked() {
                    if let Some(idx) = self.selected_output {
                        if !self.video_path_input.is_empty() {
                            let output_name = self.outputs[idx].name.clone();
                            self.send_command(IpcCommand::SetSource {
                                output: Some(output_name.clone()),
                                source: self.video_path_input.clone(),
                            });
                            self.status_message =
                                t!("msg_applying_video", output = output_name).to_string();
                        }
                    } else {
                        self.status_message = t!("sources_select_output").to_string();
                    }
                }
            });

        ui.add_space(12.0);

        // URL stream section
        egui::Frame::none()
            .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.heading(t!("sources_stream_title"));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(t!("sources_stream_url"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.url_input)
                            .desired_width(ui.available_width() - 80.0),
                    );
                });
                ui.label(t!("sources_stream_formats"));
                ui.add_space(4.0);
                if ui.button(t!("btn_apply_url")).clicked() {
                    if let Some(idx) = self.selected_output {
                        if !self.url_input.is_empty() {
                            let output_name = self.outputs[idx].name.clone();
                            self.send_command(IpcCommand::SetSource {
                                output: Some(output_name.clone()),
                                source: self.url_input.clone(),
                            });
                            self.status_message =
                                t!("msg_applying_stream", output = output_name).to_string();
                        }
                    } else {
                        self.status_message = t!("sources_select_output").to_string();
                    }
                }
            });

        ui.add_space(12.0);

        // Quick access to common paths
        egui::Frame::none()
            .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                ui.heading(t!("sources_quick_title"));
                ui.label(t!("sources_quick_hint"));
                ui.add_space(4.0);
                let common_paths = vec![
                    (
                        "~/Videos",
                        std::env::var("HOME").ok().map(|h| format!("{}/Videos", h)),
                    ),
                    (
                        "~/Pictures",
                        std::env::var("HOME")
                            .ok()
                            .map(|h| format!("{}/Pictures", h)),
                    ),
                    (
                        "~/Downloads",
                        std::env::var("HOME")
                            .ok()
                            .map(|h| format!("{}/Downloads", h)),
                    ),
                ];
                ui.horizontal(|ui| {
                    for (label, path_opt) in common_paths {
                        if let Some(path) = path_opt {
                            if ui.button(label).clicked() {
                                self.video_path_input = path;
                            }
                        }
                    }
                });
            });

        if !self.video_sources.is_empty() {
            ui.add_space(12.0);
            ui.separator();
            ui.heading(t!("sources_recent"));
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for source in &self.video_sources {
                        ui.horizontal(|ui| {
                            ui.label(&source.name);
                            ui.label(&source.path);
                        });
                    }
                });
        }
    }

    fn show_workshop_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading(t!("workshop_title"));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(t!("workshop_search"));
            ui.add(egui::TextEdit::singleline(&mut self.workshop_search).desired_width(200.0));

            if ui
                .button(if self.workshop_scan_running {
                    t!("workshop_scanning").to_string()
                } else {
                    t!("workshop_scan").to_string()
                })
                .clicked()
                && !self.workshop_scan_running
            {
                self.scan_workshop();
            }

            ui.label(t!("workshop_found", count = self.workshop_items.len()));
        });

        ui.separator();
        ui.add_space(8.0);

        if self.workshop_items.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.label(t!("workshop_empty"));
                ui.label(t!("workshop_empty_hint"));
                ui.add_space(16.0);
                ui.label(t!("workshop_requirements_title"));
                ui.label(t!("workshop_requirements_steam"));
                ui.label(t!("workshop_requirements_we"));
                ui.label(t!("workshop_requirements_subscribed"));
            });
        } else {
            // Filter items based on search (clone to avoid borrow conflicts)
            let workshop_items = self.workshop_items.clone();
            let workshop_search = self.workshop_search.clone();

            let filtered_items: Vec<_> = workshop_items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    if workshop_search.is_empty() {
                        true
                    } else {
                        item.title
                            .to_lowercase()
                            .contains(&workshop_search.to_lowercase())
                            || item.id.to_string().contains(&workshop_search)
                    }
                })
                .collect();

            ui.label(t!(
                "workshop_showing",
                shown = filtered_items.len(),
                total = workshop_items.len()
            ));

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Grid layout for Workshop items
                let num_columns = (ui.available_width() / 320.0).max(1.0) as usize;
                egui::Grid::new("workshop_grid")
                    .num_columns(num_columns)
                    .spacing([12.0, 12.0])
                    .show(ui, |ui| {
                        let mut column = 0;
                        for (idx, item) in filtered_items {
                            let is_selected = self.selected_workshop == Some(idx);

                            egui::Frame::none()
                                .fill(if is_selected {
                                    ui.style().visuals.selection.bg_fill.gamma_multiply(0.3)
                                } else {
                                    ui.style().visuals.widgets.noninteractive.bg_fill
                                })
                                .rounding(egui::Rounding::same(8.0))
                                .inner_margin(egui::Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(280.0);
                                    ui.set_max_width(280.0);

                                    // Selection state
                                    let mut selected = is_selected;
                                    if ui.checkbox(&mut selected, "").changed() {
                                        self.selected_workshop =
                                            if selected { Some(idx) } else { None };
                                    }

                                    // Item info
                                    ui.vertical(|ui| {
                                        ui.heading(&item.title);
                                        ui.label(format!("ID: {}", item.id));

                                        // Status indicator
                                        if item.is_valid && item.video_path.is_some() {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(76, 175, 80),
                                                t!("workshop_valid"),
                                            );
                                        } else {
                                            ui.colored_label(
                                                egui::Color32::from_rgb(255, 193, 7),
                                                t!("workshop_invalid"),
                                            );
                                        }

                                        ui.add_space(8.0);

                                        // Action buttons
                                        ui.horizontal(|ui| {
                                            if item.is_valid
                                                && ui.button(t!("workshop_preview")).clicked()
                                            {
                                                if let Some(ref video_path) = item.video_path {
                                                    self.video_path_input = video_path.clone();
                                                    self.selected_tab = Tab::Sources;
                                                    self.status_message = t!(
                                                        "msg_preview",
                                                        title = item.title.clone()
                                                    )
                                                    .to_string();
                                                }
                                            }

                                            if item.is_valid
                                                && ui.button(t!("workshop_import")).clicked()
                                            {
                                                self.import_workshop_item(item.id);
                                            }
                                        });

                                        // Show video path in small text
                                        if let Some(ref video_path) = item.video_path {
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new(video_path)
                                                    .small()
                                                    .color(egui::Color32::GRAY),
                                            );
                                        }
                                    });
                                });

                            column += 1;
                            if column >= num_columns {
                                ui.end_row();
                                column = 0;
                            }
                        }
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
            // Language settings (top priority for i18n)
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_language_title"));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(t!("settings_language_select"));
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

                            // Validate button
                            let validation_result = self.validate_config();
                            let (color, text) = match validation_result {
                                Ok(_) => (
                                    egui::Color32::from_rgb(76, 175, 80),
                                    t!("settings_config_valid").to_string(),
                                ),
                                Err(ref e) => (egui::Color32::from_rgb(244, 67, 54), e.clone()),
                            };
                            ui.colored_label(color, text);
                        }
                    });

                    if self.show_config_editor {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{} {}",
                                t!("settings_config_editing"),
                                self.config_path
                            ));
                            if self.config_edited {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 193, 7),
                                    t!("settings_config_unsaved"),
                                );
                            }
                        });

                        ui.separator();

                        // Text editor
                        let text_edit = egui::TextEdit::multiline(&mut self.config_content)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(15);

                        if ui.add(text_edit).changed() {
                            self.config_edited = true;
                        }

                        ui.add_space(8.0);

                        // Quick actions
                        ui.horizontal(|ui| {
                            if ui.button(t!("settings_config_reload")).clicked() {
                                self.load_config_file();
                            }

                            if ui.button(t!("settings_config_copy")).clicked() {
                                ui.output_mut(|o| o.copied_text = self.config_content.clone());
                                self.status_message = t!("settings_config_copied").to_string();
                            }

                            if ui.button(t!("settings_config_example")).clicked() {
                                self.config_content =
                                    include_str!("../../configs/config.example.yaml").to_string();
                                self.config_edited = true;
                            }
                        });
                    } else {
                        ui.label(t!("settings_config_start"));
                    }
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

                    ui.add_space(4.0);
                    let desc = match self.config_layout.as_str() {
                        "Fill" => t!("settings_video_layout_fill_desc"),
                        "Contain" => t!("settings_video_layout_contain_desc"),
                        "Stretch" => t!("settings_video_layout_stretch_desc"),
                        "Cover" => t!("settings_video_layout_cover_desc"),
                        "Centre" => t!("settings_video_layout_centre_desc"),
                        _ => "".into(),
                    };
                    ui.label(desc);

                    ui.add_space(8.0);

                    ui.checkbox(&mut self.config_loop, t!("settings_video_loop"));
                    ui.checkbox(&mut self.config_hwdec, t!("settings_video_hwdec"));

                    ui.add_space(4.0);

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

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button(t!("settings_video_apply")).clicked() {
                            if let Some(idx) = self.selected_output {
                                let output_name = self.outputs[idx].name.clone();
                                // Send config update commands
                                self.send_command(IpcCommand::ReloadConfig);
                                self.status_message =
                                    t!("msg_config_applied", output = output_name).to_string();
                            } else {
                                self.status_message = t!("sources_select_output").to_string();
                            }
                        }

                        if ui.button(t!("settings_video_export")).clicked() {
                            self.status_message = t!("msg_export_not_implemented").to_string();
                        }
                    });
                });

            ui.add_space(12.0);

            // Performance
            egui::Frame::none()
                .fill(ui.style().visuals.widgets.noninteractive.bg_fill)
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.heading(t!("settings_perf_title"));
                    ui.label(t!("settings_perf_fps"));
                    ui.label(t!("settings_perf_memory"));
                    ui.label(t!("settings_perf_decode"));
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

    fn scan_workshop(&mut self) {
        use wayvid::we::steam::SteamLibrary;
        use wayvid::we::workshop::{WorkshopScanner, WALLPAPER_ENGINE_APP_ID};

        self.workshop_scan_running = true;
        self.status_message = t!("msg_scanning_workshop").to_string();
        self.workshop_items.clear();

        match SteamLibrary::discover() {
            Ok(library) => match library.find_workshop_items(WALLPAPER_ENGINE_APP_ID) {
                Ok(paths) => match WorkshopScanner::scan(&paths) {
                    Ok(scanner) => {
                        self.workshop_items = scanner
                            .items()
                            .iter()
                            .map(|item| WorkshopItemInfo {
                                id: item.id,
                                title: item.title(),
                                path: item.path.to_string_lossy().to_string(),
                                video_path: item
                                    .video_path()
                                    .map(|p| p.to_string_lossy().to_string()),
                                is_valid: item.is_valid(),
                            })
                            .collect();
                        let valid_count = self.workshop_items.iter().filter(|i| i.is_valid).count();
                        self.status_message = t!(
                            "msg_workshop_found",
                            total = self.workshop_items.len(),
                            valid = valid_count
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

    fn import_workshop_item(&mut self, id: u64) {
        self.status_message = t!("msg_importing", id = id).to_string();

        // Find the item
        if let Some(item) = self.workshop_items.iter().find(|i| i.id == id) {
            if let Some(ref video_path) = item.video_path {
                // Set as current video source
                self.video_path_input = video_path.clone();
                self.status_message = t!("msg_imported", title = item.title.clone()).to_string();
            } else {
                self.status_message = t!("msg_import_no_video").to_string();
            }
        }
    }
}
