use eframe::egui;

mod app;
mod catalogue;
mod figcraft;
mod settings;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lucide Companion for FigCraft")
            .with_inner_size([360.0, 560.0])
            .with_min_inner_size([320.0, 440.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lucide Companion for FigCraft",
        options,
        Box::new(|cc| Ok(Box::new(app::CompanionApp::new(cc)))),
    )
}
