mod network;
mod ui;

use eframe::egui;
use network::FileTransfer;

fn main() -> eframe::Result {
    let app_name = "PS Payload Injector";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 260.0])
            .with_resizable(false),
        ..Default::default()
    };

    let inject_fn = |ip: &str, port: &str, file_path: &str| -> Result<usize, String> {
        let file_transfer =
            FileTransfer::new(ip.to_string(), port.to_string(), file_path.to_string());
        file_transfer.send_file()
    };

    eframe::run_native(
        app_name,
        options,
        Box::new(|_cc| Ok(Box::new(ui::App::new(inject_fn)))),
    )
}
