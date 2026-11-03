use eframe::egui;

mod app;
mod catalogue;
mod figcraft;
mod settings;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lucide Icon")
            .with_inner_size([520.0, 900.0])
            .with_min_inner_size([420.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lucide Icon",
        options,
        Box::new(|cc| Ok(Box::new(app::CompanionApp::new(cc)))),
    )
}
