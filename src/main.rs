#![cfg_attr(windows, windows_subsystem = "windows")]

use eframe::egui;
use ps_payload_injector::handlers::{
    create_auto_save_fn, create_auto_save_preference_fn, create_inject_fn, create_load_config_fn,
    create_reset_fn, create_save_config_fn, load_startup_config,
};

fn main() -> eframe::Result {
    let app_name = "PS Payload Injector";

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 300.0])
            .with_resizable(false),
        renderer: eframe::Renderer::Glow,
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    let inject_fn = create_inject_fn();
    let save_config_fn = create_save_config_fn();
    let load_config_fn = create_load_config_fn();
    let auto_save_fn = create_auto_save_fn();
    let auto_save_preference_fn = create_auto_save_preference_fn();
    let reset_fn = create_reset_fn();
    let startup_config = load_startup_config();

    eframe::run_native(
        app_name,
        options,
        Box::new(|_cc| {
            Ok(Box::new(ps_payload_injector::ui::App::new(
                inject_fn,
                save_config_fn,
                load_config_fn,
                auto_save_fn,
                auto_save_preference_fn,
                reset_fn,
                startup_config,
            )))
        }),
    )
}
