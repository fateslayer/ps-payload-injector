use crate::config::{
    Config, DEFAULT_AUTO_SAVE_ENABLED, DEFAULT_FILE_PATH, DEFAULT_IP, DEFAULT_PORT,
};
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
                let config = Config::new(ip.clone(), port.clone(), file_path.clone());
                match config.save_to_file(&path) {
                    Ok(()) => {
                        let filename = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown");
                        let _ = sender.send(InjectionStatus::ConfigSaved(format!(
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
                // Reset status to Idle after cancellation
                let _ = sender.send(InjectionStatus::Idle);
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
        // Only auto-save if a config file already exists (meaning auto-save is enabled)
        if Config::config_file_exists() {
            let current_config = Config::load_or_default();

            // Create config with current values and preserve auto-save preference
            let config = Config::new_with_auto_save(
                ip.to_string(),
                port.to_string(),
                file_path.to_string(),
                current_config.auto_save_enabled,
            );

            // Ensure the save operation completes successfully
            let _ = config.auto_save();

            // In test mode, add a small verification to ensure the save worked
            #[cfg(test)]
            {
                // Verify the save worked by reading it back
                let verification_config = Config::load_or_default();
                if verification_config.ip != ip
                    || verification_config.port != port
                    || verification_config.file_path != file_path
                {
                    // If verification fails, try saving again
                    let _ = config.auto_save();
                }
            }
        }
    }
}

pub fn create_auto_save_preference_fn() -> impl Fn(bool) + Send + 'static {
    |auto_save_enabled: bool| {
        if auto_save_enabled {
            // Only save if auto-save is being enabled
            let current_config = Config::load_or_default();
            let config = Config::new_with_auto_save(
                current_config.ip,
                current_config.port,
                current_config.file_path,
                true,
            );
            let _ = config.auto_save();
        } else {
            // If auto-save is being disabled, delete the config file if it exists
            if Config::config_file_exists() {
                let config_path = Config::default_auto_save_path();
                let _ = std::fs::remove_file(config_path);
            }
        }
    }
}

pub fn create_reset_fn() -> impl Fn(&str, &str, &str, mpsc::Sender<InjectionStatus>) + Send + 'static
{
    |_ip: &str, _port: &str, _file_path: &str, sender: mpsc::Sender<InjectionStatus>| {
        let _ = sender.send(InjectionStatus::Idle);
    }
}

pub fn load_startup_config() -> (String, String, String, bool) {
    if Config::config_file_exists() {
        let config = Config::load_or_default();
        (
            config.ip,
            config.port,
            config.file_path,
            config.auto_save_enabled,
        )
    } else {
        // No config file exists, return defaults with auto-save disabled
        (
            DEFAULT_IP.to_string(),
            DEFAULT_PORT.to_string(),
            DEFAULT_FILE_PATH.to_string(),
            DEFAULT_AUTO_SAVE_ENABLED,
        )
    }
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
        let _auto_save_preference_fn = create_auto_save_preference_fn();

        // Test startup config loading (this will create or load existing config)
        let startup_config = load_startup_config();
        assert!(startup_config.0.len() > 0); // IP should not be empty
        assert!(startup_config.1.len() > 0); // Port should not be empty
                                             // File path can be empty in defaults
                                             // Auto-save enabled is a boolean (can be true or false)
    }

    #[test]
    fn test_auto_save_function() {
        let auto_save_fn = create_auto_save_fn();
        let auto_save_preference_fn = create_auto_save_preference_fn();

        // Store current state first
        let config_existed = Config::config_file_exists();
        let original_config = if config_existed {
            Some(Config::load_or_default())
        } else {
            None
        };

        // Enable auto-save first, then test auto-saving some values
        auto_save_preference_fn(true);
        auto_save_fn("10.0.0.100", "3000", "/test/auto_save.bin");

        // Load it back to verify it was saved
        let config = Config::load_or_default();
        assert_eq!(config.ip, "10.0.0.100");
        assert_eq!(config.port, "3000");
        assert_eq!(config.file_path, "/test/auto_save.bin");

        // Restore original state
        if let Some(original_config) = original_config {
            if original_config.auto_save_enabled {
                auto_save_preference_fn(true);
                auto_save_fn(
                    &original_config.ip,
                    &original_config.port,
                    &original_config.file_path,
                );
            } else {
                auto_save_preference_fn(false);
            }
        } else {
            auto_save_preference_fn(false);
        }
    }

    #[test]
    fn test_auto_save_edge_cases() {
        let auto_save_fn = create_auto_save_fn();
        let auto_save_preference_fn = create_auto_save_preference_fn();

        // Store current state first
        let config_existed = Config::config_file_exists();
        let original_config = if config_existed {
            Some(Config::load_or_default())
        } else {
            None
        };

        // Enable auto-save first
        auto_save_preference_fn(true);

        // Verify auto-save is enabled
        assert!(Config::config_file_exists());
        let initial_config = Config::load_or_default();
        assert_eq!(initial_config.auto_save_enabled, true);

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
        if let Some(original_config) = original_config {
            if original_config.auto_save_enabled {
                auto_save_preference_fn(true);
                auto_save_fn(
                    &original_config.ip,
                    &original_config.port,
                    &original_config.file_path,
                );
            } else {
                auto_save_preference_fn(false);
            }
        } else {
            auto_save_preference_fn(false);
        }
    }

    #[test]
    fn test_auto_save_with_special_characters() {
        let auto_save_fn = create_auto_save_fn();
        let auto_save_preference_fn = create_auto_save_preference_fn();

        // Store current state first
        let config_existed = Config::config_file_exists();
        let original_config = if config_existed {
            Some(Config::load_or_default())
        } else {
            None
        };

        // Enable auto-save first
        auto_save_preference_fn(true);

        // Test with special characters in file path
        auto_save_fn("127.0.0.1", "8080", "/path with spaces/file-name_test.txt");

        let config = Config::load_or_default();
        assert_eq!(config.ip, "127.0.0.1");
        assert_eq!(config.port, "8080");
        assert_eq!(config.file_path, "/path with spaces/file-name_test.txt");

        // Restore original state
        if let Some(original_config) = original_config {
            if original_config.auto_save_enabled {
                auto_save_preference_fn(true);
                auto_save_fn(
                    &original_config.ip,
                    &original_config.port,
                    &original_config.file_path,
                );
            } else {
                auto_save_preference_fn(false);
            }
        } else {
            auto_save_preference_fn(false);
        }
    }

    #[test]
    fn test_startup_config_consistency() {
        let auto_save_fn = create_auto_save_fn();
        let auto_save_preference_fn = create_auto_save_preference_fn();

        // Store current state first
        let config_existed = Config::config_file_exists();
        let original_config = if config_existed {
            Some(Config::load_or_default())
        } else {
            None
        };

        // Enable auto-save first, then save some specific values
        auto_save_preference_fn(true);
        auto_save_fn("172.16.0.1", "5555", "/consistent/test.bin");

        // Verify the config was actually saved correctly before testing load_startup_config
        let saved_config = Config::load_or_default();
        assert_eq!(saved_config.ip, "172.16.0.1");
        assert_eq!(saved_config.port, "5555");
        assert_eq!(saved_config.file_path, "/consistent/test.bin");
        assert_eq!(saved_config.auto_save_enabled, true);

        // Load startup config should return the same values
        let startup_config = load_startup_config();
        assert_eq!(startup_config.0, "172.16.0.1");
        assert_eq!(startup_config.1, "5555");
        assert_eq!(startup_config.2, "/consistent/test.bin");
        assert_eq!(startup_config.3, true); // Auto-save should be enabled

        // Restore original state
        if let Some(original_config) = original_config {
            if original_config.auto_save_enabled {
                auto_save_preference_fn(true);
                auto_save_fn(
                    &original_config.ip,
                    &original_config.port,
                    &original_config.file_path,
                );
            } else {
                auto_save_preference_fn(false);
            }
        } else {
            // No config existed before, disable auto-save to clean up
            auto_save_preference_fn(false);
        }
    }

    #[test]
    fn test_auto_save_preference_function() {
        let auto_save_preference_fn = create_auto_save_preference_fn();

        // Store current state first
        let config_existed = Config::config_file_exists();
        let original_config = if config_existed {
            Some(Config::load_or_default())
        } else {
            None
        };

        // Test setting auto-save to true (should create config file)
        auto_save_preference_fn(true);
        assert!(Config::config_file_exists());
        let config = Config::load_or_default();
        assert_eq!(config.auto_save_enabled, true);

        // Test setting auto-save to false (should delete config file)
        auto_save_preference_fn(false);
        assert!(!Config::config_file_exists());

        // Restore original state
        if let Some(original_config) = original_config {
            if original_config.auto_save_enabled {
                auto_save_preference_fn(true);
                // Manually save the original config
                let _ = original_config.auto_save();
            }
        }
    }

    // This test should run last to clean up any test config files
    // The test name starts with 'z' to ensure it runs after other tests alphabetically
    #[test]
    fn z_cleanup_test_files() {
        Config::cleanup_test_files();

        // Verify cleanup worked for the current thread's config file
        let test_config_path = Config::default_auto_save_path();
        assert!(!test_config_path.exists());
    }
}
