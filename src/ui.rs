use eframe::egui;
use std::path::Path;

pub struct App<F>
where
    F: Fn(&str, &str, &str) -> Result<usize, String>,
{
    ip: String,
    port: String,
    file_path: String,
    status: String,
    inject_fn: F,
}

impl<F> App<F>
where
    F: Fn(&str, &str, &str) -> Result<usize, String>,
{
    pub fn new(inject_fn: F) -> Self {
        Self {
            ip: "192.168.1.2".to_owned(),
            port: "9025".to_owned(),
            file_path: "".to_owned(),
            status: "Idle".to_owned(),
            inject_fn,
        }
    }
}

impl<F> eframe::App for App<F>
where
    F: Fn(&str, &str, &str) -> Result<usize, String>,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);

            ui.vertical(|ui| {
                Self::add_text_input(ui, "IP Address:", &mut self.ip);
                ui.add_space(15.0);
                Self::add_text_input(ui, "Port:", &mut self.port);
                ui.add_space(15.0);
                self.add_file_input(ui);
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.add_space(90.0);
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

            ui.horizontal(|ui| {
                ui.add_sized([80.0, 20.0], egui::Label::new("Status:"));
                ui.add(
                    egui::Label::new(egui::RichText::new(&self.status).color(self.status_color()))
                        .wrap(),
                );
            });

            ui.add_space(20.0);
        });
    }
}

impl<F> App<F>
where
    F: Fn(&str, &str, &str) -> Result<usize, String>,
{
    fn add_text_input(ui: &mut egui::Ui, label: &str, text: &mut String) {
        ui.horizontal(|ui| {
            ui.add_sized([80.0, 20.0], egui::Label::new(label));
            ui.add(egui::TextEdit::singleline(text).desired_width(ui.available_width() - 20.0));
        });
    }

    fn add_file_input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_sized([80.0, 20.0], egui::Label::new("File Path:"));
            let available_width = ui.available_width() - 100.0;
            ui.add(egui::TextEdit::singleline(&mut self.file_path).desired_width(available_width));
            ui.add_space(10.0);
            if ui.button("Browse...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.file_path = path.display().to_string();
                }
            }
        });
    }

    fn status_color(&self) -> egui::Color32 {
        if self.status.starts_with("Error") {
            egui::Color32::from_rgb(220, 80, 80)
        } else if self.status.starts_with("Success") {
            egui::Color32::from_rgb(80, 180, 80)
        } else {
            egui::Color32::from_rgb(120, 120, 120)
        }
    }

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

        match (self.inject_fn)(&self.ip, &self.port, &self.file_path) {
            Ok(bytes_sent) => {
                self.status = format!("Success! Sent {} bytes", bytes_sent);
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }
}
