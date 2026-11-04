use eframe::egui;

mod app;
mod catalogue;
mod figcraft;
mod settings;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lucide Icon")
            .with_inner_size([300.0, 554.0])
            .with_min_inner_size([280.0, 500.0])
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
