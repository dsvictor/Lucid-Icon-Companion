use eframe::egui;

mod app;
mod catalogue;
mod figcraft;
mod settings;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lucide Icon")
            .with_inner_size([380.0, 700.0])
            .with_min_inner_size([340.0, 560.0])
            // On macOS, draw the app's charcoal surface behind the native traffic lights.
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false),
        ..Default::default()
    };

    eframe::run_native(
        "Lucide Icon",
        options,
        Box::new(|cc| Ok(Box::new(app::CompanionApp::new(cc)))),
    )
}
