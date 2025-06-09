use eframe::egui;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

fn main() -> eframe::Result {
    let app_name = "Payload Injector";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 260.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(app_name, options, Box::new(|_cc| Ok(Box::<App>::default())))
}

struct App {
    ip: String,
    port: String,
    file_path: String,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ip: "192.168.1.2".to_owned(),
            port: "9025".to_owned(),
            file_path: "".to_owned(),
            status: "Idle".to_owned(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            // Main content layout
            ui.vertical(|ui| {
                // IP Address row
                ui.horizontal(|ui| {
                    ui.add_sized([80.0, 20.0], egui::Label::new("IP Address:"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.ip)
                            .desired_width(ui.available_width() - 20.0),
                    );
                });

                ui.add_space(15.0);

                // Port row
                ui.horizontal(|ui| {
                    ui.add_sized([80.0, 20.0], egui::Label::new("Port:"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.port)
                            .desired_width(ui.available_width() - 20.0),
                    );
                });
                ui.add_space(15.0);

                // File Path row
                ui.horizontal(|ui| {
                    ui.add_sized([80.0, 20.0], egui::Label::new("File Path:"));
                    let available_width = ui.available_width() - 100.0; // Account for Browse button, spacing, and padding
                    ui.add(
                        egui::TextEdit::singleline(&mut self.file_path)
                            .desired_width(available_width),
                    );

                    ui.add_space(10.0);

                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.file_path = path.display().to_string();
                        }
                    }
                });
            });

            ui.add_space(20.0);

            // Action buttons
            ui.horizontal(|ui| {
                ui.add_space(90.0); // Align with the input fields
                if ui
                    .add_sized([100.0, 30.0], egui::Button::new("Inject Payload"))
                    .clicked()
                {
                    self.inject_payload();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Status section
            ui.horizontal(|ui| {
                ui.add_sized([80.0, 20.0], egui::Label::new("Status:"));
                ui.allocate_ui_with_layout(
                    [ui.available_width() - 20.0, ui.available_height()].into(),
                    egui::Layout::left_to_right(egui::Align::TOP),
                    |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(&self.status).color(
                                if self.status.starts_with("Error") {
                                    egui::Color32::from_rgb(220, 80, 80)
                                } else if self.status.starts_with("Success") {
                                    egui::Color32::from_rgb(80, 180, 80)
                                } else {
                                    egui::Color32::from_rgb(120, 120, 120)
                                },
                            ))
                            .wrap(),
                        );
                    },
                );
            });

            ui.add_space(20.0);
        });
    }
}

impl App {
    fn inject_payload(&mut self) {
        self.status = "Injecting payload...".to_string();

        if self.file_path.is_empty() {
            self.status = "Error: No file selected".to_string();
            return;
        }

        if !Path::new(&self.file_path).exists() {
            self.status = format!("Error: File does not exist: {}", self.file_path);
            return;
        }

        if self.ip.is_empty() {
            self.status = "Error: IP address is required".to_string();
            return;
        }

        if self.port.is_empty() {
            self.status = "Error: Port is required".to_string();
            return;
        }

        if self.port.parse::<u16>().is_err() {
            self.status = format!("Error: Invalid port number: {}", self.port);
            return;
        }

        self.status = format!(
            "Sending file '{}' to {}:{}",
            self.file_path, self.ip, self.port
        );

        match self.send_file_via_tcp() {
            Ok(bytes_sent) => {
                self.status = format!("Success! Sent {} bytes", bytes_sent);
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }

    fn send_file_via_tcp(&mut self) -> Result<usize, String> {
        let address = format!("{}:{}", self.ip, self.port);

        let mut file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file '{}': {}", self.file_path, e))?;

        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file '{}': {}", self.file_path, e))?;

        self.status = format!("Connecting to {}...", address);

        let mut stream = TcpStream::connect_timeout(
            &address
                .parse()
                .map_err(|e| format!("Invalid address '{}': {}", address, e))?,
            Duration::from_secs(10),
        )
        .map_err(|e| format!("Failed to connect to {}: {}", address, e))?;

        stream
            .set_write_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| format!("Failed to set write timeout: {}", e))?;

        self.status = format!("Connected! Sending {} bytes...", buffer.len());

        stream
            .write_all(&buffer)
            .map_err(|e| format!("Failed to send data: {}", e))?;

        stream
            .flush()
            .map_err(|e| format!("Failed to flush data: {}", e))?;

        Ok(buffer.len())
    }
}
