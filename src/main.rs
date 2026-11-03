use eframe::egui;

mod app;
mod catalogue;
mod figcraft;
mod settings;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lucide Companion for FigCraft")
            .with_inner_size([1080.0, 740.0])
            .with_min_inner_size([780.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lucide Companion for FigCraft",
        options,
        Box::new(|cc| Ok(Box::new(app::CompanionApp::new(cc)))),
    )
}
