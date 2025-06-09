use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub ip: String,
    pub port: String,
    pub file_path: String,
}

impl Config {
    pub fn new(ip: String, port: String, file_path: String) -> Self {
        Self {
            ip,
            port,
            file_path,
        }
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
}
