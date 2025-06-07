use eframe::egui;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

fn main() -> eframe::Result {
    let app_name = "NetCat Payload Injector";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 240.0])
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
            ui.add_space(15.0);

            ui.spacing_mut().item_spacing = egui::vec2(10.0, 12.0);
            ui.spacing_mut().button_padding = egui::vec2(8.0, 4.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(15.0, 0.0);

                ui.horizontal(|ui| {
                    ui.label("IP:");
                    ui.add_space(5.0);
                    ui.text_edit_singleline(&mut self.ip);
                });

                ui.add_space(30.0);

                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.add_space(5.0);
                    ui.text_edit_singleline(&mut self.port);
                });
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);

                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.add_space(5.0);
                    ui.text_edit_singleline(&mut self.file_path);
                });

                ui.add_space(15.0);

                if ui.button("Open file..").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.file_path = path.display().to_string();
                    }
                }
            });

            ui.add_space(15.0);

            ui.horizontal(|ui| {
                if ui.button("Inject").clicked() {
                    self.inject_payload();
                }
            });

            ui.add_space(10.0);

            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.add_space(5.0);
                ui.colored_label(
                    if self.status.starts_with("Error") {
                        egui::Color32::from_rgb(200, 100, 100)
                    } else if self.status.starts_with("Success") {
                        egui::Color32::from_rgb(100, 200, 100)
                    } else {
                        egui::Color32::from_rgb(150, 150, 150)
                    },
                    &self.status,
                );
            });

            ui.add_space(15.0);
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
