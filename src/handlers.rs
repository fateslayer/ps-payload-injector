use crate::config::Config;
use crate::network::FileTransfer;
use crate::ui::InjectionStatus;
use std::sync::mpsc;

pub fn create_inject_fn(
) -> impl Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static {
    |ip: &str, port: &str, file_path: &str, sender: mpsc::Sender<InjectionStatus>| {
        let ip = ip.to_string();
        let port = port.to_string();
        let file_path = file_path.to_string();

        // Spawn the async task in a separate thread
        std::thread::spawn(move || {
            // Create tokio runtime for async operations
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = sender.send(InjectionStatus::Error(format!(
                        "Failed to create async runtime: {}",
                        e
                    )));
                    return;
                }
            };

            // Execute the async task
            rt.block_on(async move {
                // Extract filename for display
                let filename = std::path::Path::new(&file_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");

                // Send status update: Reading file
                let _ = sender.send(InjectionStatus::InProgress(format!(
                    "Reading file '{}'...",
                    filename
                )));

                // Small delay to show the status
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                let file_transfer = FileTransfer::new(ip.clone(), port.clone(), file_path.clone());

                // Send status update: Connecting
                let _ = sender.send(InjectionStatus::InProgress(format!(
                    "Connecting to {}:{}...",
                    ip, port
                )));

                // Small delay to show the status
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

                // Send status update: Sending data
                let _ = sender.send(InjectionStatus::InProgress(format!(
                    "Sending '{}' to {}:{}...",
                    filename, ip, port
                )));

                match file_transfer.send_file().await {
                    Ok(bytes_sent) => {
                        let _ = sender.send(InjectionStatus::Success(bytes_sent));
                    }
                    Err(e) => {
                        let _ = sender.send(InjectionStatus::Error(e));
                    }
                }
            });
        });
    }
}

pub fn create_save_config_fn(
) -> impl Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static {
    |ip: &str, port: &str, file_path: &str, sender: mpsc::Sender<InjectionStatus>| {
        let ip = ip.to_string();
        let port = port.to_string();
        let file_path = file_path.to_string();

        // Spawn the save config task in a separate thread
        std::thread::spawn(move || {
            // Send status update: Preparing to save
            let _ = sender.send(InjectionStatus::InProgress(
                "Preparing to save config...".to_string(),
            ));

            // Create file dialog
            let mut dialog = rfd::FileDialog::new()
                .add_filter("JSON files", &["json"])
                .set_file_name("config.json");

            // Set current directory as default
            if let Ok(current_dir) = std::env::current_dir() {
                dialog = dialog.set_directory(&current_dir);
            }

            if let Some(path) = dialog.save_file() {
                let config = Config::new(ip, port, file_path);

                match config.save_to_file(&path) {
                    Ok(()) => {
                        let filename = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown");
                        let _ = sender.send(InjectionStatus::InProgress(format!(
                            "Config saved to '{}'",
                            filename
                        )));
                    }
                    Err(e) => {
                        let _ = sender.send(InjectionStatus::Error(format!(
                            "Failed to save config: {}",
                            e
                        )));
                    }
                }
            } else {
                let _ = sender.send(InjectionStatus::InProgress(
                    "Config save cancelled".to_string(),
                ));
            }
        });
    }
}

pub fn create_load_config_fn() -> impl Fn(mpsc::Sender<InjectionStatus>) + Send + 'static {
    |sender: mpsc::Sender<InjectionStatus>| {
        // Spawn the load config task in a separate thread
        std::thread::spawn(move || {
            // Send status update: Preparing to load
            let _ = sender.send(InjectionStatus::InProgress(
                "Preparing to load config...".to_string(),
            ));

            // Create file dialog for loading
            let mut dialog = rfd::FileDialog::new().add_filter("JSON files", &["json"]);

            // Set current directory as default
            if let Ok(current_dir) = std::env::current_dir() {
                dialog = dialog.set_directory(&current_dir);
            }

            if let Some(path) = dialog.pick_file() {
                let _ = sender.send(InjectionStatus::InProgress(
                    "Loading config file...".to_string(),
                ));

                match Config::load_from_file(&path) {
                    Ok(config) => {
                        // Validate the loaded config
                        if config.ip.trim().is_empty() {
                            let _ = sender.send(InjectionStatus::Error(
                                "Invalid config: IP address is empty".to_string(),
                            ));
                            return;
                        }

                        if config.port.parse::<u16>().is_err() {
                            let _ = sender.send(InjectionStatus::Error(format!(
                                "Invalid config: Invalid port number '{}'",
                                config.port
                            )));
                            return;
                        }

                        // Note: We don't validate file_path existence here as user might want to load config with non-existent files

                        let _ = sender.send(InjectionStatus::ConfigLoaded(
                            config.ip,
                            config.port,
                            config.file_path,
                        ));
                    }
                    Err(e) => {
                        let _ = sender.send(InjectionStatus::Error(format!(
                            "Failed to load config: {}",
                            e
                        )));
                    }
                }
            } else {
                let _ = sender.send(InjectionStatus::InProgress(
                    "Config loading cancelled".to_string(),
                ));
            }
        });
    }
}

pub fn create_auto_save_fn() -> impl Fn(&str, &str, &str) + Send + 'static {
    |ip: &str, port: &str, file_path: &str| {
        // Create config from current values and save it silently
        let config = Config::new(ip.to_string(), port.to_string(), file_path.to_string());
        let _ = config.auto_save(); // Silent save - ignore errors for auto-save
    }
}

pub fn load_startup_config() -> (String, String, String) {
    let config = Config::load_or_default();
    (config.ip, config.port, config.file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_functions_creation() {
        // Test that all handler functions can be created without panicking
        let _inject_fn = create_inject_fn();
        let _save_config_fn = create_save_config_fn();
        let _load_config_fn = create_load_config_fn();
        let _auto_save_fn = create_auto_save_fn();

        // Test startup config loading (this will create or load existing config)
        let startup_config = load_startup_config();
        assert!(startup_config.0.len() > 0); // IP should not be empty
        assert!(startup_config.1.len() > 0); // Port should not be empty
                                             // File path can be empty in defaults
    }

    #[test]
    fn test_auto_save_edge_cases() {
        let auto_save_fn = create_auto_save_fn();

        // Store current state first
        let original_config = Config::load_or_default();

        // Test with empty values
        auto_save_fn("", "", "");
        let config = Config::load_or_default();
        assert_eq!(config.ip, "");
        assert_eq!(config.port, "");
        assert_eq!(config.file_path, "");

        // Test with maximum port value
        auto_save_fn("255.255.255.255", "65535", "/max/test.bin");
        let config = Config::load_or_default();
        assert_eq!(config.ip, "255.255.255.255");
        assert_eq!(config.port, "65535");
        assert_eq!(config.file_path, "/max/test.bin");

        // Restore original state
        auto_save_fn(
            &original_config.ip,
            &original_config.port,
            &original_config.file_path,
        );
    }

    #[test]
    fn test_startup_config_consistency() {
        let auto_save_fn = create_auto_save_fn();

        // Store current state first
        let original_config = Config::load_or_default();

        // Save some specific values
        auto_save_fn("172.16.0.1", "5555", "/consistent/test.bin");

        // Load startup config should return the same values
        let startup_config = load_startup_config();
        assert_eq!(startup_config.0, "172.16.0.1");
        assert_eq!(startup_config.1, "5555");
        assert_eq!(startup_config.2, "/consistent/test.bin");

        // Restore original state
        auto_save_fn(
            &original_config.ip,
            &original_config.port,
            &original_config.file_path,
        );
    }

    #[test]
    fn test_auto_save_with_special_characters() {
        let auto_save_fn = create_auto_save_fn();

        // Store current state first
        let original_config = Config::load_or_default();

        // Test with special characters in file path
        auto_save_fn("127.0.0.1", "8080", "/path with spaces/file-name_test.txt");

        let config = Config::load_or_default();
        assert_eq!(config.ip, "127.0.0.1");
        assert_eq!(config.port, "8080");
        assert_eq!(config.file_path, "/path with spaces/file-name_test.txt");

        // Restore original state
        auto_save_fn(
            &original_config.ip,
            &original_config.port,
            &original_config.file_path,
        );
    }

    #[test]
    fn test_auto_save_function() {
        let auto_save_fn = create_auto_save_fn();

        // Store current state first
        let original_config = Config::load_or_default();

        // Test auto-saving some values
        auto_save_fn("10.0.0.100", "3000", "/test/auto_save.bin");

        // Load it back to verify it was saved
        let config = Config::load_or_default();
        assert_eq!(config.ip, "10.0.0.100");
        assert_eq!(config.port, "3000");
        assert_eq!(config.file_path, "/test/auto_save.bin");

        // Restore original state
        auto_save_fn(
            &original_config.ip,
            &original_config.port,
            &original_config.file_path,
        );
    }
}
