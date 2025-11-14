//! wayvid GUI Control Panel

use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use wayvid::ctl::ipc_client::IpcClient;
use wayvid::ctl::protocol::{IpcCommand, IpcResponse};

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("wayvid_gui=debug,wayvid=debug")
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("wayvid Control Panel"),
        ..Default::default()
    };

    eframe::run_native(
        "wayvid",
        options,
        Box::new(|_cc| Ok(Box::new(WayvidApp::default()))),
    )
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
    selected_source: Option<usize>,
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
        Self {
            ipc_tx: None,
            ipc_rx: None,
            selected_tab: Tab::Outputs,
            outputs: Vec::new(),
            video_sources: Vec::new(),
            workshop_items: Vec::new(),
            selected_output: None,
            selected_source: None,
            selected_workshop: None,
            video_path_input: String::new(),
            url_input: String::new(),
            workshop_search: String::new(),
            config_layout: "Fill".to_string(),
            config_volume: 0.5,
            config_mute: true,
            config_loop: true,
            config_hwdec: true,
            status_message: "Not connected".to_string(),
            connection_status: ConnectionStatus::Disconnected,
            workshop_scan_running: false,
        }
    }
}

impl eframe::App for WayvidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for IPC responses
        self.poll_responses();

        // Request repaint for continuous updates
        ctx.request_repaint();

        // Top panel - Title and status
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎬 wayvid Control Panel");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Connection status indicator
                    let (color, text) = match self.connection_status {
                        ConnectionStatus::Connected => (egui::Color32::GREEN, "● Connected"),
                        ConnectionStatus::Connecting => (egui::Color32::YELLOW, "● Connecting..."),
                        ConnectionStatus::Disconnected => (egui::Color32::GRAY, "● Disconnected"),
                        ConnectionStatus::Error => (egui::Color32::RED, "● Error"),
                    };
                    ui.colored_label(color, text);

                    if ui.button("🔄 Refresh").clicked() {
                        self.refresh_outputs();
                    }

                    if self.connection_status == ConnectionStatus::Disconnected
                        && ui.button("📡 Connect").clicked()
                    {
                        self.connect_ipc();
                    }
                });
            });
        });

        // Bottom panel - Status bar
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
            });
        });

        // Left panel - Navigation tabs
        egui::SidePanel::left("side_panel")
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Navigation");
                ui.separator();

                ui.selectable_value(&mut self.selected_tab, Tab::Outputs, "📺 Outputs");
                ui.selectable_value(&mut self.selected_tab, Tab::Sources, "🎬 Video Sources");
                ui.selectable_value(&mut self.selected_tab, Tab::Workshop, "🎮 Workshop");
                ui.selectable_value(&mut self.selected_tab, Tab::Settings, "⚙ Settings");
            });

        // Central panel - Main content
        egui::CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tab::Outputs => self.show_outputs_tab(ui),
            Tab::Sources => self.show_sources_tab(ui),
            Tab::Workshop => self.show_workshop_tab(ui),
            Tab::Settings => self.show_settings_tab(ui),
        });
    }
}

impl WayvidApp {
    fn connect_ipc(&mut self) {
        self.connection_status = ConnectionStatus::Connecting;
        self.status_message = "Connecting to wayvid...".to_string();

        // Check if wayvid is running
        if !IpcClient::is_running() {
            self.connection_status = ConnectionStatus::Error;
            self.status_message =
                "Error: wayvid daemon not running. Start with 'wayvid run'".to_string();
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
        self.status_message = "Connected to wayvid daemon".to_string();

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
                self.status_message = format!("Error sending command: {}", e);
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
                            format!("Status updated - {} outputs", self.outputs.len());
                    } else {
                        self.status_message = "Command successful".to_string();
                    }
                } else {
                    self.status_message = "Command successful".to_string();
                }
            }
            IpcResponse::Error { message } => {
                self.status_message = format!("Error: {}", message);
                self.connection_status = ConnectionStatus::Error;
            }
        }
    }

    fn refresh_outputs(&mut self) {
        if self.connection_status == ConnectionStatus::Connected {
            self.send_command(IpcCommand::GetStatus);
            self.status_message = "Refreshing outputs...".to_string();
        } else {
            self.status_message = "Not connected. Click 'Connect' first.".to_string();
        }
    }

    fn show_outputs_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Output Management");
        ui.separator();

        if self.outputs.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("No outputs detected");
                ui.label("Click 'Connect' to detect displays");
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let outputs = self.outputs.clone(); // Clone to avoid borrow conflicts
            for (idx, output) in outputs.iter().enumerate() {
                let is_selected = self.selected_output == Some(idx);

                ui.group(|ui| {
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
                                ui.colored_label(egui::Color32::GREEN, "● Active");
                            } else {
                                ui.colored_label(egui::Color32::GRAY, "○ Inactive");
                            }
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if output.active {
                                if ui.button("⏸ Pause").clicked() {
                                    self.send_command(IpcCommand::Pause {
                                        output: Some(output.name.clone()),
                                    });
                                }
                            } else if ui.button("▶ Resume").clicked() {
                                self.send_command(IpcCommand::Resume {
                                    output: Some(output.name.clone()),
                                });
                            }

                            if ui.button("⚙ Configure").clicked() {
                                self.status_message =
                                    format!("Configuring output: {}", output.name);
                            }
                        });
                    });
                });

                ui.add_space(5.0);
            }
        });
    }

    fn show_sources_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Video Sources");
        ui.separator();

        // Local file section
        ui.group(|ui| {
            ui.heading("📁 Local File");
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.text_edit_singleline(&mut self.video_path_input);
                if ui.button("Browse...").clicked() {
                    // Native file dialog support would require rfd crate
                    self.status_message = "Hint: Drag & drop files or paste path".to_string();
                }
            });
            ui.horizontal(|ui| {
                if ui.button("✓ Apply to Selected Output").clicked() {
                    if let Some(idx) = self.selected_output {
                        if !self.video_path_input.is_empty() {
                            let output_name = self.outputs[idx].name.clone();
                            self.send_command(IpcCommand::SetSource {
                                output: Some(output_name.clone()),
                                source: self.video_path_input.clone(),
                            });
                            self.status_message = format!("Applying video to {}", output_name);
                        }
                    } else {
                        self.status_message = "Please select an output first".to_string();
                    }
                }
            });
        });

        ui.add_space(10.0);

        // URL stream section
        ui.group(|ui| {
            ui.heading("🌐 Stream URL");
            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(&mut self.url_input);
            });
            ui.label("Supports: HTTP(S), RTSP, HLS, DASH");
            ui.horizontal(|ui| {
                if ui.button("✓ Apply URL to Selected Output").clicked() {
                    if let Some(idx) = self.selected_output {
                        if !self.url_input.is_empty() {
                            let output_name = self.outputs[idx].name.clone();
                            self.send_command(IpcCommand::SetSource {
                                output: Some(output_name.clone()),
                                source: self.url_input.clone(),
                            });
                            self.status_message = format!("Applying stream to {}", output_name);
                        }
                    } else {
                        self.status_message = "Please select an output first".to_string();
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Quick access to common paths
        ui.group(|ui| {
            ui.heading("📂 Quick Access");
            ui.label("Common video directories:");
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
            for (label, path_opt) in common_paths {
                if let Some(path) = path_opt {
                    if ui.button(label).clicked() {
                        self.video_path_input = path;
                    }
                }
            }
        });

        if !self.video_sources.is_empty() {
            ui.add_space(10.0);
            ui.separator();
            ui.heading("Recent Sources");
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
        ui.heading("Steam Workshop");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.text_edit_singleline(&mut self.workshop_search);

            if ui
                .button(if self.workshop_scan_running {
                    "⏳ Scanning..."
                } else {
                    "🔄 Scan Workshop"
                })
                .clicked()
                && !self.workshop_scan_running
            {
                self.scan_workshop();
            }

            ui.label(format!("Found: {} items", self.workshop_items.len()));
        });

        ui.add_space(10.0);

        if self.workshop_items.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("No Workshop items found");
                ui.label("Click 'Scan Workshop' to search for Wallpaper Engine items");
                ui.add_space(20.0);
                ui.label("Make sure you have:");
                ui.label("  • Steam installed");
                ui.label("  • Wallpaper Engine in your library");
                ui.label("  • Workshop items subscribed");
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

            ui.label(format!(
                "Showing {} of {} items",
                filtered_items.len(),
                workshop_items.len()
            ));

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Grid layout for Workshop items
                let num_columns = (ui.available_width() / 300.0).max(1.0) as usize;
                egui::Grid::new("workshop_grid")
                    .num_columns(num_columns)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        let mut column = 0;
                        for (idx, item) in filtered_items {
                            let is_selected = self.selected_workshop == Some(idx);

                            ui.group(|ui| {
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
                                            egui::Color32::GREEN,
                                            "✓ Valid video wallpaper",
                                        );
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::YELLOW,
                                            "⚠ No video or invalid",
                                        );
                                    }

                                    ui.add_space(5.0);

                                    // Action buttons
                                    ui.horizontal(|ui| {
                                        if item.is_valid && ui.button("▶ Preview").clicked() {
                                            if let Some(ref video_path) = item.video_path {
                                                self.video_path_input = video_path.clone();
                                                self.selected_tab = Tab::Sources;
                                                self.status_message =
                                                    format!("Preview: {}", item.title);
                                            }
                                        }

                                        if item.is_valid && ui.button("📥 Import").clicked() {
                                            self.import_workshop_item(item.id);
                                        }
                                    });

                                    // Show video path in small text
                                    if let Some(ref video_path) = item.video_path {
                                        ui.add_space(3.0);
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
        ui.heading("Settings");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Video Configuration
            ui.group(|ui| {
                ui.heading("🎬 Video Configuration");

                ui.horizontal(|ui| {
                    ui.label("Layout Mode:");
                    egui::ComboBox::from_id_salt("layout_mode")
                        .selected_text(&self.config_layout)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.config_layout,
                                "Fill".to_string(),
                                "Fill (recommended)",
                            );
                            ui.selectable_value(
                                &mut self.config_layout,
                                "Contain".to_string(),
                                "Contain",
                            );
                            ui.selectable_value(
                                &mut self.config_layout,
                                "Stretch".to_string(),
                                "Stretch",
                            );
                            ui.selectable_value(
                                &mut self.config_layout,
                                "Cover".to_string(),
                                "Cover",
                            );
                            ui.selectable_value(
                                &mut self.config_layout,
                                "Centre".to_string(),
                                "Centre",
                            );
                        });
                });

                ui.add_space(5.0);
                match self.config_layout.as_str() {
                    "Fill" => ui.label("↔ Scale to cover screen, crop edges (no black bars)"),
                    "Contain" => ui.label("↔ Fit inside screen (may have black bars)"),
                    "Stretch" => ui.label("↔ Stretch to fill (may distort)"),
                    "Cover" => ui.label("↔ Alias for Fill mode"),
                    "Centre" => ui.label("↔ Original size, centered"),
                    _ => ui.label(""),
                };

                ui.add_space(10.0);

                ui.checkbox(&mut self.config_loop, "Loop playback");
                ui.checkbox(&mut self.config_hwdec, "Hardware decoding (VA-API/NVDEC)");

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.config_mute, "Mute");
                    if !self.config_mute {
                        ui.label("Volume:");
                        let volume_pct = (self.config_volume * 100.0) as i32;
                        ui.add(
                            egui::Slider::new(&mut self.config_volume, 0.0..=1.0)
                                .text(format!("{}%", volume_pct)),
                        );
                    }
                });

                ui.add_space(10.0);

                if ui.button("💾 Apply to Selected Output").clicked() {
                    if let Some(idx) = self.selected_output {
                        let output_name = self.outputs[idx].name.clone();
                        // Send config update commands
                        self.send_command(IpcCommand::ReloadConfig);
                        self.status_message = format!("Config applied to {}", output_name);
                    } else {
                        self.status_message = "Please select an output first".to_string();
                    }
                }

                if ui.button("📋 Save as Config File").clicked() {
                    self.status_message = "Config export not implemented yet".to_string();
                }
            });

            ui.add_space(10.0);

            // Application Settings
            ui.group(|ui| {
                ui.heading("⚙ Application");
                ui.label("These settings are for future use:");
                let mut auto_connect = false;
                let mut tray_icon = false;
                let mut notifications = true;
                ui.checkbox(&mut auto_connect, "Auto-connect on startup");
                ui.checkbox(&mut tray_icon, "Show system tray icon");
                ui.checkbox(&mut notifications, "Enable notifications");
            });

            ui.add_space(10.0);

            // Performance
            ui.group(|ui| {
                ui.heading("⚡ Performance");
                ui.label("Max FPS: Unlimited (vsync)");
                ui.label("Memory limit: 100 MB (default)");
                ui.label("Decode mode: Shared (optimal)");
            });

            ui.add_space(10.0);

            // About
            ui.group(|ui| {
                ui.heading("ℹ About");
                ui.label(format!("wayvid v{}", env!("CARGO_PKG_VERSION")));
                ui.label("Dynamic video wallpaper engine for Wayland");
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.hyperlink_to("🔗 GitHub", "https://github.com/YangYuS8/wayvid");
                    ui.hyperlink_to(
                        "📖 Documentation",
                        "https://github.com/YangYuS8/wayvid/tree/main/docs",
                    );
                });
            });
        });
    }

    fn scan_workshop(&mut self) {
        use wayvid::we::steam::SteamLibrary;
        use wayvid::we::workshop::{WorkshopScanner, WALLPAPER_ENGINE_APP_ID};

        self.workshop_scan_running = true;
        self.status_message = "Scanning Workshop items...".to_string();
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
                        self.status_message = format!(
                            "✓ Found {} Workshop items ({} valid)",
                            self.workshop_items.len(),
                            self.workshop_items.iter().filter(|i| i.is_valid).count()
                        );
                    }
                    Err(e) => {
                        self.status_message = format!("Scanner error: {}", e);
                    }
                },
                Err(e) => {
                    self.status_message = format!("Workshop scan error: {}", e);
                }
            },
            Err(e) => {
                self.status_message = format!("Steam not found: {}", e);
            }
        }

        self.workshop_scan_running = false;
    }

    fn import_workshop_item(&mut self, id: u64) {
        self.status_message = format!("Importing Workshop item {}...", id);

        // Find the item
        if let Some(item) = self.workshop_items.iter().find(|i| i.id == id) {
            if let Some(ref video_path) = item.video_path {
                // Set as current video source
                self.video_path_input = video_path.clone();
                self.status_message = format!(
                    "✓ Imported: {} - Apply to an output in Sources tab",
                    item.title
                );
            } else {
                self.status_message =
                    "Error: No video file found in this Workshop item".to_string();
            }
        }
    }
}
