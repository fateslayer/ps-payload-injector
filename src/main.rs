mod config;
mod network;
mod ui;

use config::Config;
use eframe::egui;
use network::FileTransfer;
use std::sync::mpsc;
use ui::InjectionStatus;

fn main() -> eframe::Result {
    let app_name = "PS Payload Injector";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };

    let inject_fn =
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

                    let file_transfer =
                        FileTransfer::new(ip.clone(), port.clone(), file_path.clone());

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
        };

    let save_config_fn =
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
        };

    eframe::run_native(
        app_name,
        options,
        Box::new(|_cc| Ok(Box::new(ui::App::new(inject_fn, save_config_fn)))),
    )
}
