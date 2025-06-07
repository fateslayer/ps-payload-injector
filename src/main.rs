use eframe::egui;

fn main() -> eframe::Result {
    let app_name = "NetCat Payload Injector";

    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(app_name, options, Box::new(|_cc| Ok(Box::<App>::default())))
}

struct App {
    ip: String,
    port: String,
    file_path: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ip: "192.168.1.2".to_owned(),
            port: "9025".to_owned(),
            file_path: "".to_owned(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label("IP:");
                    ui.text_edit_singleline(&mut self.ip);
                });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.port);
                });
            });

            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.text_edit_singleline(&mut self.file_path);
                });

                ui.add_space(20.0);

                if ui.button("Open file..").clicked() {
                    println!("Opening file..");
                }
            })
        });
    }
}
