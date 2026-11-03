use crate::{
    catalogue::{self, Icon},
    figcraft::FigCraftClient,
    settings::Settings,
};
use eframe::egui;
use std::{
    fs,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub struct CompanionApp {
    settings: Settings,
    query: String,
    icons: Vec<Icon>,
    selected: Option<String>,
    status: String,
    loading: bool,
    tx: Sender<Result<(Vec<Icon>, String), String>>,
    rx: Receiver<Result<(Vec<Icon>, String), String>>,
    preview: Option<(String, egui::TextureHandle)>,
}

impl CompanionApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let icons = catalogue::load_cache(&settings.cache_dir);
        let status = if icons.is_empty() {
            "Lucide catalogue not installed. Choose Update catalogue to download the complete upstream set.".to_owned()
        } else {
            format!("Loaded {} cached Lucide icons.", icons.len())
        };
        let (tx, rx) = mpsc::channel();

        Self {
            settings,
            query: String::new(),
            icons,
            selected: None,
            status,
            loading: false,
            tx,
            rx,
            preview: None,
        }
    }

    fn update_catalogue(&mut self, ctx: &egui::Context) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.status = "Downloading the latest lucide-static package…".into();
        let sender = self.tx.clone();
        let cache = self.settings.cache_dir.clone();
        let repaint = ctx.clone();

        thread::spawn(move || {
            let result = catalogue::download_latest(&cache).map_err(|e| e.to_string());
            let _ = sender.send(result);
            repaint.request_repaint();
        });
    }

    fn selected_icon(&self) -> Option<Icon> {
        let name = self.selected.as_ref()?;
        self.icons.iter().find(|icon| &icon.name == name).cloned()
    }

    fn styled_svg(&self, icon: &Icon) -> String {
        icon.svg
            .replace("currentColor", &self.settings.stroke_color)
            .replace(
                "stroke-width=\"2\"",
                &format!("stroke-width=\"{}\"", self.settings.stroke_width),
            )
            .replace(
                "fill=\"none\"",
                &format!("fill=\"{}\"", self.settings.fill_color),
            )
    }

    fn export_svg(&mut self, name: &str, svg: &str) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("{name}.svg"))
            .add_filter("SVG image", &["svg"])
            .save_file()
        else {
            return;
        };

        self.status = match fs::write(&path, svg) {
            Ok(()) => format!("Saved {}", path.display()),
            Err(error) => format!("Could not save SVG: {error}"),
        };
    }

    fn launch_figcraft(&mut self) {
        self.status = match FigCraftClient::new(self.settings.control_port).launch() {
            Ok(()) => format!(
                "Launching FigCraft with control enabled on port {}…",
                self.settings.control_port
            ),
            Err(error) => error.to_string(),
        };
    }

    fn insert_into_figcraft(&mut self, icon: &Icon) {
        let svg = self.styled_svg(icon);
        self.status = match FigCraftClient::new(self.settings.control_port)
            .insert_svg(&icon.name, &svg)
        {
            Ok(result) => format!("Inserted {} into FigCraft. {result}", icon.name),
            Err(error) => error.to_string(),
        };
    }

    fn render_preview(
        ctx: &egui::Context,
        svg: &str,
        key: &str,
    ) -> Option<egui::TextureHandle> {
        let tree =
            resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).ok()?;
        let source_size = tree.size().to_int_size();
        let scale = 4.0_f32;
        let width = source_size.width().max(1) * scale as u32;
        let height = source_size.height().max(1) * scale as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;

        resvg::render(
            &tree,
            tiny_skia::Transform::from_scale(scale, scale),
            &mut pixmap.as_mut(),
        );

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize],
            pixmap.data(),
        );
        Some(ctx.load_texture(key, image, egui::TextureOptions::LINEAR))
    }
}

impl eframe::App for CompanionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(result) = self.rx.try_recv() {
            self.loading = false;
            match result {
                Ok((icons, version)) => {
                    self.icons = icons;
                    self.selected = None;
                    self.preview = None;
                    self.status = format!(
                        "Lucide {version}: {} SVG icons cached.",
                        self.icons.len()
                    );
                }
                Err(error) => self.status = format!("Catalogue update failed: {error}"),
            }
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Lucide Companion");
                ui.label("for FigCraft");
                ui.separator();
                if ui
                    .add_enabled(!self.loading, egui::Button::new("Update catalogue"))
                    .clicked()
                {
                    self.update_catalogue(ctx);
                }
                if ui.button("Launch FigCraft").clicked() {
                    self.launch_figcraft();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("by Jayanta");
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} icons", self.icons.len()));
                });
            });
        });

        egui::SidePanel::left("icon_browser")
            .resizable(true)
            .default_width(290.0)
            .show(ctx, |ui| {
                ui.heading("Icon library");
                ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search icon names…")
                        .desired_width(f32::INFINITY),
                );
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let matches = catalogue::search(&self.icons, &self.query);
                    for icon in matches {
                        let name = icon.name.clone();
                        if ui
                            .selectable_label(self.selected.as_ref() == Some(&name), &name)
                            .clicked()
                        {
                            self.selected = Some(name);
                            self.preview = None;
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(icon) = self.selected_icon() {
                ui.heading(&icon.name);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.settings.icon_size, 24.0..=256.0)
                            .text("Preview size"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.settings.stroke_width, 0.5..=4.0)
                            .text("Stroke"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Stroke");
                    ui.text_edit_singleline(&mut self.settings.stroke_color);
                    ui.label("Fill");
                    ui.text_edit_singleline(&mut self.settings.fill_color);
                });

                let svg = self.styled_svg(&icon);
                let key = format!(
                    "{}-{}-{}-{}-{}",
                    icon.name,
                    self.settings.icon_size,
                    self.settings.stroke_width,
                    self.settings.stroke_color,
                    self.settings.fill_color
                );

                if self.preview.as_ref().map(|(cached, _)| cached) != Some(&key) {
                    self.preview =
                        Self::render_preview(ctx, &svg, &key).map(|texture| (key.clone(), texture));
                }

                ui.add_space(12.0);
                if let Some((_, texture)) = &self.preview {
                    let size = egui::Vec2::splat(self.settings.icon_size);
                    ui.add(egui::Image::new((texture.id(), size)));
                } else {
                    ui.label("SVG preview could not be rendered.");
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Insert into FigCraft").clicked() {
                        self.insert_into_figcraft(&icon);
                    }
                    if ui.button("Export SVG…").clicked() {
                        self.export_svg(&icon.name, &svg);
                    }
                    if ui.button("Copy SVG").clicked() {
                        ctx.copy_text(svg.clone());
                        self.status = format!("Copied {} SVG to clipboard.", icon.name);
                    }
                    if ui.button("Save preferences").clicked() {
                        self.status = match self.settings.save() {
                            Ok(()) => "Preferences saved.".into(),
                            Err(error) => format!("Could not save preferences: {error}"),
                        };
                    }
                });

                ui.collapsing("SVG source", |ui| {
                    let mut source = svg.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut source)
                            .desired_rows(10)
                            .code_editor()
                            .desired_width(f32::INFINITY),
                    );
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select an icon to preview, customize, export or insert it.");
                });
            }
        });
    }
}
