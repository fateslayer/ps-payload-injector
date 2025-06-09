use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

// Default configuration values
pub const DEFAULT_IP: &str = "192.168.1.4";
pub const DEFAULT_PORT: &str = "9025";
pub const DEFAULT_FILE_PATH: &str = "";
pub const DEFAULT_AUTO_SAVE_ENABLED: bool = false;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ip: String,
    pub port: String,
    pub file_path: String,
    pub auto_save_enabled: bool,
}

impl Config {
    pub fn new(ip: String, port: String, file_path: String) -> Self {
        Self {
            ip,
            port,
            file_path,
            auto_save_enabled: DEFAULT_AUTO_SAVE_ENABLED,
        }
    }

    pub fn new_with_auto_save(
        ip: String,
        port: String,
        file_path: String,
        auto_save_enabled: bool,
    ) -> Self {
        Self {
            ip,
            port,
            file_path,
            auto_save_enabled,
        }
    }

    /// Create a config with default values
    pub fn default() -> Self {
        Self::new(
            DEFAULT_IP.to_string(),
            DEFAULT_PORT.to_string(),
            DEFAULT_FILE_PATH.to_string(),
        )
    }

    /// Clean up test config files (only available in test builds)
    #[cfg(test)]
    pub fn cleanup_test_files() {
        use std::fs;

        // Clean up the current thread's test config file
        let current_test_config = Self::default_auto_save_path();
        if current_test_config.exists() {
            let _ = fs::remove_file(&current_test_config);
        }

        // Also clean up any other test-prefixed config files that might exist
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.starts_with("test-app_config") && filename.ends_with(".json") {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    /// Get the default path for auto-save config file
    pub fn default_auto_save_path() -> PathBuf {
        // Use different filenames for tests vs production
        if cfg!(test) {
            // Use thread-specific test config files to avoid test interference
            use std::thread;
            let thread_id = format!("{:?}", thread::current().id());
            // Sanitize the thread ID to be filename-safe
            let safe_thread_id = thread_id.replace("ThreadId(", "").replace(")", "");
            PathBuf::from(format!("test-app_config-{}.json", safe_thread_id))
        } else {
            PathBuf::from("app_config.json")
        }
    }

    /// Load config from default auto-save path with fallback to defaults
    /// Does not create a config file if none exists
    pub fn load_or_default() -> Self {
        let config_path = Self::default_auto_save_path();

        match Self::load_from_file(&config_path) {
            Ok(config) => config,
            Err(_) => {
                // Return default config if loading fails - do NOT save anything
                Self::default()
            }
        }
    }

    /// Auto-save config to default location
    pub fn auto_save(&self) -> Result<(), String> {
        let config_path = Self::default_auto_save_path();
        self.save_to_file(&config_path)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        let mut file = File::create(path.as_ref()).map_err(|e| {
            format!(
                "Failed to create config file '{}': {}",
                path.as_ref().display(),
                e
            )
        })?;

        file.write_all(json.as_bytes()).map_err(|e| {
            format!(
                "Failed to write config file '{}': {}",
                path.as_ref().display(),
                e
            )
        })?;

        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            format!(
                "Failed to read config file '{}': {}",
                path.as_ref().display(),
                e
            )
        })?;

        let config: Config = serde_json::from_str(&file_content).map_err(|e| {
            format!(
                "Failed to parse config file '{}': {}",
                path.as_ref().display(),
                e
            )
        })?;

        Ok(config)
    }

    /// Check if auto-save config file exists
    pub fn config_file_exists() -> bool {
        let config_path = Self::default_auto_save_path();
        config_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_new() {
        let config = Config::new(
            "192.168.1.1".to_string(),
            "8080".to_string(),
            "/path/to/file".to_string(),
        );

        assert_eq!(config.ip, "192.168.1.1");
        assert_eq!(config.port, "8080");
        assert_eq!(config.file_path, "/path/to/file");
        assert_eq!(config.auto_save_enabled, DEFAULT_AUTO_SAVE_ENABLED); // Should use constant
    }

    #[test]
    fn test_config_new_with_auto_save() {
        let config = Config::new_with_auto_save(
            "192.168.1.1".to_string(),
            "8080".to_string(),
            "/path/to/file".to_string(),
            true,
        );

        assert_eq!(config.ip, "192.168.1.1");
        assert_eq!(config.port, "8080");
        assert_eq!(config.file_path, "/path/to/file");
        assert_eq!(config.auto_save_enabled, true);
    }

    #[test]
    fn test_save_and_load_config() {
        let original_config = Config::new_with_auto_save(
            "10.0.0.1".to_string(),
            "9025".to_string(),
            "/test/path/payload.bin".to_string(),
            true,
        );

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Test saving
        assert!(original_config.save_to_file(temp_path).is_ok());

        // Test loading
        let loaded_config = Config::load_from_file(temp_path).expect("Failed to load config");

        assert_eq!(loaded_config.ip, original_config.ip);
        assert_eq!(loaded_config.port, original_config.port);
        assert_eq!(loaded_config.file_path, original_config.file_path);
        assert_eq!(
            loaded_config.auto_save_enabled,
            original_config.auto_save_enabled
        );
    }

    #[test]
    fn test_save_config_creates_valid_json() {
        let config = Config::new(
            "172.16.0.1".to_string(),
            "3000".to_string(),
            "/home/user/file.txt".to_string(),
        );

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        assert!(config.save_to_file(temp_path).is_ok());

        let file_content = fs::read_to_string(temp_path).expect("Failed to read file");
        assert!(file_content.contains("\"ip\": \"172.16.0.1\""));
        assert!(file_content.contains("\"port\": \"3000\""));
        assert!(file_content.contains("\"file_path\": \"/home/user/file.txt\""));
    }

    #[test]
    fn test_load_config_invalid_json() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Write invalid JSON
        fs::write(temp_path, "{ invalid json }").expect("Failed to write file");

        let result = Config::load_from_file(temp_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse config file"));
    }

    #[test]
    fn test_load_config_missing_fields() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Write JSON with missing fields
        fs::write(temp_path, r#"{"ip": "192.168.1.1"}"#).expect("Failed to write file");

        let result = Config::load_from_file(temp_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse config file"));
    }

    #[test]
    fn test_load_config_nonexistent_file() {
        let result = Config::load_from_file("/path/that/does/not/exist.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read config file"));
    }

    #[test]
    fn test_save_config_to_readonly_location() {
        let config = Config::new(
            "192.168.1.1".to_string(),
            "8080".to_string(),
            "/test/path".to_string(),
        );

        // Try to save to a location that doesn't exist and can't be created
        let result = config.save_to_file("/root/readonly/config.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to create config file"));
    }

    #[test]
    fn test_config_with_special_characters() {
        let config = Config::new(
            "192.168.1.1".to_string(),
            "8080".to_string(),
            "/path with spaces/file name.txt".to_string(),
        );

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        assert!(config.save_to_file(temp_path).is_ok());
        let loaded_config = Config::load_from_file(temp_path).expect("Failed to load config");

        assert_eq!(loaded_config.file_path, "/path with spaces/file name.txt");
    }

    #[test]
    fn test_config_with_empty_values() {
        let config = Config::new("".to_string(), "".to_string(), "".to_string());

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        assert!(config.save_to_file(temp_path).is_ok());
        let loaded_config = Config::load_from_file(temp_path).expect("Failed to load config");

        assert_eq!(loaded_config.ip, "");
        assert_eq!(loaded_config.port, "");
        assert_eq!(loaded_config.file_path, "");
    }

    #[test]
    fn test_load_or_default() {
        // Test loading defaults when no config file exists
        let config = Config::load_or_default();

        // Should have default values
        assert_eq!(config.ip, DEFAULT_IP);
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.file_path, DEFAULT_FILE_PATH);
        assert_eq!(config.auto_save_enabled, DEFAULT_AUTO_SAVE_ENABLED);
    }

    #[test]
    fn test_auto_save() {
        let config = Config::new(
            "10.0.0.1".to_string(),
            "8080".to_string(),
            "/test/path.txt".to_string(),
        );

        // Auto-save should work without errors
        // Note: This creates app_config.json in the current directory
        assert!(config.auto_save().is_ok());

        // Load it back to verify
        let loaded = Config::load_or_default();
        assert_eq!(loaded.ip, "10.0.0.1");
        assert_eq!(loaded.port, "8080");
        assert_eq!(loaded.file_path, "/test/path.txt");

        // Clean up - restore original state by checking what was there before
        // Since tests may run in any order, let's just clean up our specific test values
        if loaded.ip == "10.0.0.1" && loaded.port == "8080" {
            let default_config = Config::default();
            let _ = default_config.auto_save();
        }
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert_eq!(config.ip, DEFAULT_IP);
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.file_path, DEFAULT_FILE_PATH);
        assert_eq!(config.auto_save_enabled, DEFAULT_AUTO_SAVE_ENABLED);
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
