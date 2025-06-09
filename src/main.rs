mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let app_name = "PS Payload Injector";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 260.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        app_name,
        options,
        Box::new(|_cc| Ok(Box::<ui::App>::default())),
    )
}
