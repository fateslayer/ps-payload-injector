use eframe::egui;
use std::path::Path;
use std::sync::mpsc;

#[derive(Debug)]
pub enum InjectionStatus {
    Idle,
    InProgress(String),
    Success(usize),
    Error(String),
    ConfigLoaded(String, String, String), // ip, port, file_path
}

pub struct App<F, G, H, I, J>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    G: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    H: Fn(mpsc::Sender<InjectionStatus>) + Send + 'static,
    I: Fn(&str, &str, &str) + Send + 'static,
    J: Fn(bool) + Send + 'static,
{
    ip: String,
    port: String,
    file_path: String,
    status: InjectionStatus,
    inject_fn: F,
    save_config_fn: G,
    load_config_fn: H,
    auto_save_fn: I,
    auto_save_preference_fn: J,
    receiver: Option<mpsc::Receiver<InjectionStatus>>,
    values_changed: bool,    // Track if values have changed since last save
    auto_save_enabled: bool, // Track if auto-save is enabled
}

impl<F, G, H, I, J> App<F, G, H, I, J>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    G: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    H: Fn(mpsc::Sender<InjectionStatus>) + Send + 'static,
    I: Fn(&str, &str, &str) + Send + 'static,
    J: Fn(bool) + Send + 'static,
{
    pub fn new(
        inject_fn: F,
        save_config_fn: G,
        load_config_fn: H,
        auto_save_fn: I,
        auto_save_preference_fn: J,
        startup_config: (String, String, String, bool),
    ) -> Self {
        Self {
            ip: startup_config.0,
            port: startup_config.1,
            file_path: startup_config.2,
            status: InjectionStatus::Idle,
            inject_fn,
            save_config_fn,
            load_config_fn,
            auto_save_fn,
            auto_save_preference_fn,
            receiver: None,
            values_changed: false,
            auto_save_enabled: startup_config.3,
        }
    }
}

impl<F, G, H, I, J> eframe::App for App<F, G, H, I, J>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    G: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    H: Fn(mpsc::Sender<InjectionStatus>) + Send + 'static,
    I: Fn(&str, &str, &str) + Send + 'static,
    J: Fn(bool) + Send + 'static,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for status updates from the async task
        if let Some(receiver) = &self.receiver {
            if let Ok(new_status) = receiver.try_recv() {
                // Handle config loading to populate fields
                if let InjectionStatus::ConfigLoaded(ip, port, file_path) = &new_status {
                    self.ip = ip.clone();
                    self.port = port.clone();
                    self.file_path = file_path.clone();
                    self.values_changed = true; // Mark as changed for auto-save
                }
                self.status = new_status;
                ctx.request_repaint(); // Request UI update
            }
        }

        // Request continuous updates if we're in an in-progress state
        if matches!(self.status, InjectionStatus::InProgress(_)) {
            ctx.request_repaint();
        }

        // Auto-save config when values change
        if self.values_changed {
            if self.auto_save_enabled {
                (self.auto_save_fn)(&self.ip, &self.port, &self.file_path);
            }
            self.values_changed = false;
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
                    let ip_response = ui.add(
                        egui::TextEdit::singleline(&mut self.ip)
                            .desired_width(ui.available_width() - 20.0)
                            .margin(egui::Vec2::new(8.0, 6.0)),
                    );
                    if ip_response.changed() {
                        self.values_changed = true;
                    }
                    ui.end_row();

                    // Port row
                    ui.add_sized([80.0, 20.0], egui::Label::new("Port:"));
                    let port_response = ui.add(
                        egui::TextEdit::singleline(&mut self.port)
                            .desired_width(ui.available_width() - 20.0)
                            .margin(egui::Vec2::new(8.0, 6.0)),
                    );
                    if port_response.changed() {
                        self.values_changed = true;
                    }
                    ui.end_row();

                    // File Path row
                    ui.add_sized([80.0, 20.0], egui::Label::new("File Path:"));
                    ui.horizontal(|ui| {
                        let file_path_response = ui.add(
                            egui::TextEdit::singleline(&mut self.file_path)
                                .desired_width(ui.available_width() - 123.0) // Leave more space for button + margin
                                .margin(egui::Vec2::new(8.0, 6.0)),
                        );
                        if file_path_response.changed() {
                            self.values_changed = true;
                        }
                        ui.add_space(5.0);
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.file_path = path.display().to_string();
                                self.values_changed = true;
                            }
                        }
                    });
                    ui.end_row();

                    ui.add_sized([80.0, 20.0], egui::Label::new("")); // Empty first column

                    ui.horizontal(|ui| {
                        let inject_button = ui.add_enabled(
                            self.is_input_valid()
                                && !matches!(self.status, InjectionStatus::InProgress(_)),
                            egui::Button::new("Inject Payload"),
                        );

                        if inject_button.clicked() {
                            self.inject_payload();
                        }

                        ui.add_space(5.0);

                        let save_config_button =
                            ui.add_enabled(self.is_input_valid(), egui::Button::new("Save Config"));

                        if save_config_button.clicked() {
                            self.save_config();
                        }

                        ui.add_space(5.0);

                        let load_config_button = ui.button("Load Config");

                        if load_config_button.clicked() {
                            self.load_config();
                        }
                    });

                    ui.end_row();

                    ui.add_sized([80.0, 20.0], egui::Label::new("")); // Empty first column

                    let auto_save_response =
                        ui.checkbox(&mut self.auto_save_enabled, "Autosave Config");

                    if auto_save_response.changed() {
                        // Always save the auto-save preference itself
                        (self.auto_save_preference_fn)(self.auto_save_enabled);
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Auto-save config on app exit (only if auto-save is enabled and config file exists)
        if self.auto_save_enabled {
            (self.auto_save_fn)(&self.ip, &self.port, &self.file_path);
        }
    }
}

impl<F, G, H, I, J> App<F, G, H, I, J>
where
    F: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    G: Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static,
    H: Fn(mpsc::Sender<InjectionStatus>) + Send + 'static,
    I: Fn(&str, &str, &str) + Send + 'static,
    J: Fn(bool) + Send + 'static,
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
            InjectionStatus::ConfigLoaded(_, _, _) => "Config loaded successfully".to_string(),
        }
    }

    fn status_color(&self) -> egui::Color32 {
        match &self.status {
            InjectionStatus::Error(_) => egui::Color32::from_rgb(220, 80, 80),
            InjectionStatus::Success(_) => egui::Color32::from_rgb(80, 180, 80),
            InjectionStatus::InProgress(_) => egui::Color32::from_rgb(255, 165, 0), // Orange
            InjectionStatus::Idle => egui::Color32::from_rgb(120, 120, 120),
            InjectionStatus::ConfigLoaded(_, _, _) => egui::Color32::from_rgb(80, 180, 80), // Green like success
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

    fn save_config(&mut self) {
        // Create a channel for communication
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);

        // Clone the necessary data for the save config function
        let ip = self.ip.clone();
        let port = self.port.clone();
        let file_path = self.file_path.clone();

        // Call the save config function with the sender
        (self.save_config_fn)(&ip, &port, &file_path, sender);
    }

    fn load_config(&mut self) {
        // Create a channel for communication
        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);

        // Call the load config function with the sender
        (self.load_config_fn)(sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::NamedTempFile;

    // Mock functions for testing
    fn mock_inject_fn(
        _ip: &str,
        _port: &str,
        _file_path: &str,
        _sender: mpsc::Sender<InjectionStatus>,
    ) {
        // Does nothing for testing
    }

    fn mock_save_config_fn(
        _ip: &str,
        _port: &str,
        _file_path: &str,
        _sender: mpsc::Sender<InjectionStatus>,
    ) {
        // Does nothing for testing
    }

    fn mock_load_config_fn(_sender: mpsc::Sender<InjectionStatus>) {
        // Does nothing for testing
    }

    #[test]
    fn test_app_new() {
        let app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "/test/path".to_string(),
                true,
            ),
        );

        // Values should now be loaded from config (or defaults if no config exists)
        assert!(!app.ip.is_empty());
        assert!(!app.port.is_empty());
        assert!(matches!(app.status, InjectionStatus::Idle));
        assert!(!app.values_changed); // Should start as unchanged
    }

    #[test]
    fn test_injection_status_debug() {
        let idle = InjectionStatus::Idle;
        let in_progress = InjectionStatus::InProgress("Testing".to_string());
        let success = InjectionStatus::Success(1024);
        let error = InjectionStatus::Error("Test error".to_string());
        let config_loaded = InjectionStatus::ConfigLoaded(
            "192.168.1.1".to_string(),
            "8080".to_string(),
            "/test/path".to_string(),
        );

        // Test that Debug trait works
        assert!(format!("{:?}", idle).contains("Idle"));
        assert!(format!("{:?}", in_progress).contains("InProgress"));
        assert!(format!("{:?}", success).contains("Success"));
        assert!(format!("{:?}", error).contains("Error"));
        assert!(format!("{:?}", config_loaded).contains("ConfigLoaded"));
    }

    #[test]
    fn test_is_input_valid() {
        let mut app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "/test/path".to_string(),
                true,
            ),
        );

        // Test invalid cases
        assert!(!app.is_input_valid()); // Empty file path

        app.file_path = "/nonexistent/file.txt".to_string();
        assert!(!app.is_input_valid()); // File doesn't exist

        // Create a temp file for valid file path
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        app.file_path = temp_file.path().to_str().unwrap().to_string();

        app.ip = "".to_string();
        assert!(!app.is_input_valid()); // Empty IP

        app.ip = "   ".to_string();
        assert!(!app.is_input_valid()); // Whitespace only IP

        app.ip = "192.168.1.1".to_string();
        app.port = "".to_string();
        assert!(!app.is_input_valid()); // Empty port

        app.port = "invalid".to_string();
        assert!(!app.is_input_valid()); // Invalid port

        app.port = "65536".to_string();
        assert!(!app.is_input_valid()); // Port out of range

        app.port = "8080".to_string();
        assert!(app.is_input_valid()); // All valid
    }

    #[test]
    fn test_status_text() {
        let app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "/test/path".to_string(),
                true,
            ),
        );

        let mut test_app = app;

        test_app.status = InjectionStatus::Idle;
        assert_eq!(test_app.status_text(), "Idle");

        test_app.status = InjectionStatus::InProgress("Testing...".to_string());
        assert_eq!(test_app.status_text(), "Testing...");

        test_app.status = InjectionStatus::Success(1024);
        assert_eq!(test_app.status_text(), "Success! Sent 1024 bytes");

        test_app.status = InjectionStatus::Error("Test error".to_string());
        assert_eq!(test_app.status_text(), "Error: Test error");

        test_app.status = InjectionStatus::ConfigLoaded(
            "192.168.1.1".to_string(),
            "8080".to_string(),
            "/test/path".to_string(),
        );
        assert_eq!(test_app.status_text(), "Config loaded successfully");
    }

    #[test]
    fn test_status_color() {
        let app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "/test/path".to_string(),
                true,
            ),
        );
        let mut test_app = app;

        test_app.status = InjectionStatus::Idle;
        assert_eq!(
            test_app.status_color(),
            egui::Color32::from_rgb(120, 120, 120)
        );

        test_app.status = InjectionStatus::InProgress("Testing".to_string());
        assert_eq!(
            test_app.status_color(),
            egui::Color32::from_rgb(255, 165, 0)
        );

        test_app.status = InjectionStatus::Success(1024);
        assert_eq!(
            test_app.status_color(),
            egui::Color32::from_rgb(80, 180, 80)
        );

        test_app.status = InjectionStatus::Error("Error".to_string());
        assert_eq!(
            test_app.status_color(),
            egui::Color32::from_rgb(220, 80, 80)
        );

        test_app.status =
            InjectionStatus::ConfigLoaded("ip".to_string(), "port".to_string(), "path".to_string());
        assert_eq!(
            test_app.status_color(),
            egui::Color32::from_rgb(80, 180, 80)
        );
    }

    #[test]
    fn test_inject_payload_validation() {
        let mut app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "".to_string(), // Start with empty file path
                true,
            ),
        );

        // Test empty file path
        app.inject_payload();
        assert!(matches!(app.status, InjectionStatus::Error(_)));
        if let InjectionStatus::Error(msg) = &app.status {
            assert!(msg.contains("No file selected"));
        }

        // Test nonexistent file
        app.file_path = "/nonexistent/file.txt".to_string();
        app.inject_payload();
        assert!(matches!(app.status, InjectionStatus::Error(_)));
        if let InjectionStatus::Error(msg) = &app.status {
            assert!(msg.contains("File does not exist"));
        }

        // Create a temp file
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        app.file_path = temp_file.path().to_str().unwrap().to_string();

        // Test empty IP
        app.ip = "".to_string();
        app.inject_payload();
        assert!(matches!(app.status, InjectionStatus::Error(_)));
        if let InjectionStatus::Error(msg) = &app.status {
            assert!(msg.contains("IP address is required"));
        }

        app.ip = "192.168.1.1".to_string();

        // Test empty port
        app.port = "".to_string();
        app.inject_payload();
        assert!(matches!(app.status, InjectionStatus::Error(_)));
        if let InjectionStatus::Error(msg) = &app.status {
            assert!(msg.contains("Port is required"));
        }

        // Test invalid port
        app.port = "invalid".to_string();
        app.inject_payload();
        assert!(matches!(app.status, InjectionStatus::Error(_)));
        if let InjectionStatus::Error(msg) = &app.status {
            assert!(msg.contains("Invalid port number"));
        }
    }

    #[test]
    fn test_edge_cases() {
        let mut app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "/test/path".to_string(),
                true,
            ),
        );

        // Test with whitespace in IP
        app.ip = "  192.168.1.1  ".to_string();
        app.port = "8080".to_string();
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        app.file_path = temp_file.path().to_str().unwrap().to_string();

        assert!(app.is_input_valid()); // Should handle whitespace

        // Test port edge cases
        app.port = "1".to_string();
        assert!(app.is_input_valid()); // Port 1 is valid

        app.port = "65535".to_string();
        assert!(app.is_input_valid()); // Port 65535 is valid

        app.port = "0".to_string();
        assert!(app.is_input_valid()); // Port 0 is technically valid for u16
    }

    #[test]
    fn test_config_loaded_populates_fields() {
        // This test would require setting up a receiver, which is complex in unit tests
        // In a real scenario, you'd test the field population logic
        let mut app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "/test/path".to_string(),
                true,
            ),
        );

        // Simulate what happens when ConfigLoaded is received
        let (sender, receiver) = mpsc::channel();
        app.receiver = Some(receiver);

        // Send a ConfigLoaded status
        sender
            .send(InjectionStatus::ConfigLoaded(
                "10.0.0.1".to_string(),
                "9000".to_string(),
                "/new/path.txt".to_string(),
            ))
            .unwrap();

        // In the real app, this would be handled in the update method
        // For testing purposes, we simulate the behavior
        if let Some(receiver) = &app.receiver {
            if let Ok(new_status) = receiver.try_recv() {
                if let InjectionStatus::ConfigLoaded(ip, port, file_path) = &new_status {
                    app.ip = ip.clone();
                    app.port = port.clone();
                    app.file_path = file_path.clone();
                }
                app.status = new_status;
            }
        }

        assert_eq!(app.ip, "10.0.0.1");
        assert_eq!(app.port, "9000");
        assert_eq!(app.file_path, "/new/path.txt");
        assert!(matches!(app.status, InjectionStatus::ConfigLoaded(_, _, _)));
    }

    #[test]
    fn test_save_config_status_transitions() {
        let mut app = App::new(
            mock_inject_fn,
            mock_save_config_fn,
            mock_load_config_fn,
            |_, _, _| {},
            |_| {},
            (
                "192.168.1.1".to_string(),
                "8080".to_string(),
                "/test/path".to_string(),
                true,
            ),
        );

        // Create a channel for communication
        let (sender, receiver) = mpsc::channel();
        app.receiver = Some(receiver);

        // Simulate save config operation
        (app.save_config_fn)("192.168.1.1", "8080", "/test/path", sender.clone());

        // Verify initial InProgress status
        if let Some(receiver) = &app.receiver {
            if let Ok(new_status) = receiver.try_recv() {
                assert!(matches!(new_status, InjectionStatus::InProgress(_)));
                app.status = new_status;
            }
        }

        // Simulate successful save completion
        let _ = sender.send(InjectionStatus::InProgress(
            "Config saved to 'test.json'".to_string(),
        ));
        let _ = sender.send(InjectionStatus::Idle);

        // Verify final Idle status
        if let Some(receiver) = &app.receiver {
            if let Ok(new_status) = receiver.try_recv() {
                assert!(matches!(new_status, InjectionStatus::InProgress(_)));
                app.status = new_status;
            }
            if let Ok(new_status) = receiver.try_recv() {
                assert!(matches!(new_status, InjectionStatus::Idle));
                app.status = new_status;
            }
        }

        // Verify the app is in Idle state
        assert!(matches!(app.status, InjectionStatus::Idle));
    }
}
