use eframe::egui;
use ps_payload_injector::handlers::{
    create_inject_fn, create_load_config_fn, create_save_config_fn,
};

fn main() -> eframe::Result {
    let app_name = "PS Payload Injector";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([483.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };

    let inject_fn = create_inject_fn();
    let save_config_fn = create_save_config_fn();
    let load_config_fn = create_load_config_fn();

    eframe::run_native(
        app_name,
        options,
        Box::new(|_cc| {
            Ok(Box::new(ps_payload_injector::ui::App::new(
                inject_fn,
                save_config_fn,
                load_config_fn,
            )))
        }),
    )
}
