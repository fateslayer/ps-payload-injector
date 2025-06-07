use eframe::egui;

fn main() -> eframe::Result {
    let app_name = "NetCat Payload Injector";

    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_simple_native(app_name, options, update_fun)
}

fn update_fun(ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Hello World");
    });
}
