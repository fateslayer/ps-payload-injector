use eframe::egui;
use std::path::Path;
use std::sync::mpsc;

#[derive(Debug)]
pub enum InjectionStatus {
    Idle,
    InProgress(String),
    Success(usize),
    Error(String),
}

pub struct App<F>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
{
    ip: String,
    port: String,
    file_path: String,
    status: InjectionStatus,
    inject_fn: F,
    receiver: Option<mpsc::Receiver<InjectionStatus>>,
}

impl<F> App<F>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
{
    pub fn new(inject_fn: F) -> Self {
        Self {
            ip: "192.168.1.2".to_owned(),
            port: "9025".to_owned(),
            file_path: "".to_owned(),
            status: InjectionStatus::Idle,
            inject_fn,
            receiver: None,
        }
    }
}

impl<F> eframe::App for App<F>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for status updates from the async task
        if let Some(receiver) = &self.receiver {
            if let Ok(new_status) = receiver.try_recv() {
                self.status = new_status;
                ctx.request_repaint(); // Request UI update
            }
        }

        // Increase font sizes for all text elements and add padding
        ctx.style_mut(|style| {
            style
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap()
                .size = 16.0;
            style
                .text_styles
                .get_mut(&egui::TextStyle::Button)
                .unwrap()
                .size = 16.0;
            style
                .text_styles
                .get_mut(&egui::TextStyle::Heading)
                .unwrap()
                .size = 20.0;
            style
                .text_styles
                .get_mut(&egui::TextStyle::Monospace)
                .unwrap()
                .size = 16.0;
            style
                .text_styles
                .get_mut(&egui::TextStyle::Small)
                .unwrap()
                .size = 14.0;

            // Add padding to buttons and text inputs
            style.spacing.button_padding = egui::Vec2::new(12.0, 5.0);
            style.spacing.indent = 20.0;
            style.spacing.item_spacing = egui::Vec2::new(8.0, 8.0);
            style.spacing.interact_size.y = 30.0;
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);

            // IP Address and Port rows - 2 columns
            egui::Grid::new("basic_input_grid")
                .num_columns(2)
                .spacing([15.0, 15.0])
                .show(ui, |ui| {
                    // IP Address row
                    ui.add_sized([80.0, 20.0], egui::Label::new("IP Address:"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.ip)
                            .desired_width(ui.available_width() - 20.0)
                            .margin(egui::Vec2::new(8.0, 6.0)),
                    );
                    ui.end_row();

                    // Port row
                    ui.add_sized([80.0, 20.0], egui::Label::new("Port:"));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.port)
                            .desired_width(ui.available_width() - 20.0)
                            .margin(egui::Vec2::new(8.0, 6.0)),
                    );
                    ui.end_row();

                    // File Path row
                    ui.add_sized([80.0, 20.0], egui::Label::new("File Path:"));
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.file_path)
                                .desired_width(ui.available_width() - 123.0) // Leave more space for button + margin
                                .margin(egui::Vec2::new(8.0, 6.0)),
                        );
                        ui.add_space(5.0);
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.file_path = path.display().to_string();
                            }
                        }
                    });
                    ui.end_row();

                    ui.add_sized([80.0, 20.0], egui::Label::new("")); // Empty first column
                    let inject_button = ui.add_enabled(
                        self.is_input_valid()
                            && !matches!(self.status, InjectionStatus::InProgress(_)),
                        egui::Button::new(match &self.status {
                            InjectionStatus::InProgress(_) => "Injecting...",
                            _ => "Inject Payload",
                        }),
                    );

                    if inject_button.clicked() {
                        self.inject_payload();
                    }
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.separator();

            egui::Grid::new("status_grid")
                .num_columns(2)
                .spacing([15.0, 15.0])
                .show(ui, |ui| {
                    ui.add_sized([80.0, 20.0], egui::Label::new("Status:"));
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&self.status_text()).color(self.status_color()),
                        )
                        .wrap(),
                    );
                    ui.end_row();
                });
        });
    }
}

impl<F> App<F>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
{
    fn is_input_valid(&self) -> bool {
        // Check if IP address is not empty and not just whitespace
        if self.ip.trim().is_empty() {
            return false;
        }

        // Check if port is not empty, not just whitespace, and is a valid u16
        if self.port.trim().is_empty() || self.port.parse::<u16>().is_err() {
            return false;
        }

        // Check if file path is not empty and file exists
        if self.file_path.trim().is_empty() || !Path::new(&self.file_path).exists() {
            return false;
        }

        true
    }

    fn status_text(&self) -> String {
        match &self.status {
            InjectionStatus::Idle => "Idle".to_string(),
            InjectionStatus::InProgress(msg) => msg.clone(),
            InjectionStatus::Success(bytes) => format!("Success! Sent {} bytes", bytes),
            InjectionStatus::Error(msg) => format!("Error: {}", msg),
        }
    }

    fn status_color(&self) -> egui::Color32 {
        match &self.status {
            InjectionStatus::Error(_) => egui::Color32::from_rgb(220, 80, 80),
            InjectionStatus::Success(_) => egui::Color32::from_rgb(80, 180, 80),
            InjectionStatus::InProgress(_) => egui::Color32::from_rgb(255, 165, 0), // Orange
            InjectionStatus::Idle => egui::Color32::from_rgb(120, 120, 120),
        }
    }

    fn inject_payload(&mut self) {
        self.status = InjectionStatus::InProgress("Preparing injection...".to_string());

        if self.file_path.is_empty() {
            self.status = InjectionStatus::Error("No file selected".to_string());
            return;
        }

        if !Path::new(&self.file_path).exists() {
            self.status =
                InjectionStatus::Error(format!("File does not exist: {}", self.file_path));
            return;
        }

        if self.ip.is_empty() {
            self.status = InjectionStatus::Error("IP address is required".to_string());
            return;
        }

        if self.port.is_empty() {
            self.status = InjectionStatus::Error("Port is required".to_string());
            return;
        }

        if self.port.parse::<u16>().is_err() {
            self.status = InjectionStatus::Error(format!("Invalid port number: {}", self.port));
            return;
        }

        // Create a channel for communication
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);

        // Clone the necessary data for the async task
        let ip = self.ip.clone();
        let port = self.port.clone();
        let file_path = self.file_path.clone();

        // Call the injection function with the sender
        (self.inject_fn)(&ip, &port, &file_path, sender);
    }
}
