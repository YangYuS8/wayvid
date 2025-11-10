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
    selected_output: Option<usize>,
    #[allow(dead_code)] // Reserved for future video source selection feature
    selected_source: Option<usize>,

    // Status
    status_message: String,
    connection_status: ConnectionStatus,
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
            selected_output: None,
            selected_source: None,
            status_message: "Not connected".to_string(),
            connection_status: ConnectionStatus::Disconnected,
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
            self.status_message = "Error: wayvid daemon not running. Start with 'wayvid run'".to_string();
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
    fn ipc_thread(cmd_rx: Receiver<IpcCommand>, resp_tx: Sender<IpcResponse>) -> anyhow::Result<()> {
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
                    if let Ok(status) = serde_json::from_value::<wayvid::ctl::protocol::DaemonStatus>(value) {
                        self.outputs = status.outputs.into_iter().map(|o| OutputInfo {
                            name: o.name,
                            width: o.width as u32,
                            height: o.height as u32,
                            active: o.playing && !o.paused,
                        }).collect();
                        self.status_message = format!("Status updated - {} outputs", self.outputs.len());
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

        ui.horizontal(|ui| {
            if ui.button("📁 Add Local File").clicked() {
                // TODO: Open file dialog
                self.status_message = "File dialog not implemented yet".to_string();
            }

            if ui.button("🌐 Add URL").clicked() {
                self.status_message = "URL input not implemented yet".to_string();
            }
        });

        ui.add_space(10.0);

        if self.video_sources.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("No video sources");
                ui.label("Add a local file or URL to get started");
            });
        } else {
            // Show video sources grid
            ui.label("Video sources list (TODO)");
        }
    }

    fn show_workshop_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Steam Workshop");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.text_edit_singleline(&mut String::new());

            if ui.button("Scan Workshop").clicked() {
                self.scan_workshop();
            }
        });

        ui.add_space(10.0);

        ui.label("Workshop integration coming soon...");
        ui.label("Features:");
        ui.label("  • Browse installed Workshop items");
        ui.label("  • Preview with thumbnails");
        ui.label("  • One-click import");
    }

    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();

        ui.label("Application settings:");
        ui.checkbox(&mut true, "Auto-connect on startup");
        ui.checkbox(&mut false, "Show system tray icon");
        ui.checkbox(&mut true, "Enable notifications");

        ui.add_space(20.0);

        ui.label("About:");
        ui.label(format!("wayvid v{}", env!("CARGO_PKG_VERSION")));
        ui.hyperlink_to("GitHub", "https://github.com/YangYuS8/wayvid");
    }

    fn scan_workshop(&mut self) {
        use wayvid::we::steam::SteamLibrary;

        self.status_message = "Scanning Workshop items...".to_string();

        match SteamLibrary::discover() {
            Ok(library) => {
                match library.find_workshop_items(wayvid::we::workshop::WALLPAPER_ENGINE_APP_ID) {
                    Ok(items) => {
                        self.status_message = format!("Found {} Workshop items", items.len());
                    }
                    Err(e) => {
                        self.status_message = format!("Workshop scan error: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Steam not found: {}", e);
            }
        }
    }
}
