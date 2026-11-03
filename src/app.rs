use crate::{catalogue::{self, Icon}, figcraft::FigCraftClient, settings::Settings};
use eframe::egui;
use std::{fs, sync::mpsc::{self, Receiver, Sender}, thread};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryView { All, Favorites, Recent }

enum BackgroundResult {
    Connection(Result<String, String>),
    Import(Result<String, String>),
}

pub struct CompanionApp {
    settings: Settings,
    query: String,
    icons: Vec<Icon>,
    selected: Option<String>,
    view: LibraryView,
    group: Option<String>,
    status: String,
    loading: bool,
    importing: bool,
    catalogue_tx: Sender<Result<(Vec<Icon>, String), String>>,
    catalogue_rx: Receiver<Result<(Vec<Icon>, String), String>>,
    action_tx: Sender<BackgroundResult>,
    action_rx: Receiver<BackgroundResult>,
    preview: Option<(String, egui::TextureHandle)>,
    grid_previews: std::collections::HashMap<String, egui::TextureHandle>,
}

fn safe_hex_color(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") { return None; }
    let valid_length = matches!(value.len(), 4 | 7 | 9);
    if valid_length && value.starts_with('#') && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Some(value)
    } else {
        None
    }
}

impl CompanionApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let icons = catalogue::load_cache(&settings.cache_dir);
        let status = if icons.is_empty() {
            "Catalogue not installed. Choose Update catalogue to download all Lucide icons.".to_owned()
        } else {
            format!("Ready · {} cached icons", icons.len())
        };
        let (catalogue_tx, catalogue_rx) = mpsc::channel();
        let (action_tx, action_rx) = mpsc::channel();
        Self {
            settings, query: String::new(), icons, selected: None, view: LibraryView::All,
            group: None, status, loading: false, importing: false,
            catalogue_tx, catalogue_rx, action_tx, action_rx, preview: None,
            grid_previews: std::collections::HashMap::new(),
        }
    }

    fn save_settings(&mut self) {
        self.status = match self.settings.save() {
            Ok(()) => "Preferences saved.".into(),
            Err(error) => format!("Could not save preferences: {error}"),
        };
    }

    fn update_catalogue(&mut self, ctx: &egui::Context) {
        if self.loading { return; }
        self.loading = true;
        self.status = "Downloading the latest Lucide package and tag metadata…".into();
        let sender = self.catalogue_tx.clone();
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
        let stroke = safe_hex_color(&self.settings.stroke_color).unwrap_or("#000000");
        let fill = safe_hex_color(&self.settings.fill_color);
        let svg = icon.svg.replace("currentColor", stroke)
            .replace("stroke-width=\"2\"", &format!("stroke-width=\"{}\"", if self.settings.stroke_width.is_finite() { self.settings.stroke_width.clamp(0.5, 4.0) } else { 2.0 }));
        match fill {
            Some(color) => svg.replace("fill=\"none\"", &format!("fill=\"{color}\"")),
            None => svg,
        }
    }

    fn select_icon(&mut self, name: &str) {
        self.selected = Some(name.to_owned());
        self.settings.remember(name);
        let _ = self.settings.save();
        self.preview = None;
    }

    fn filtered_icon_names(&self) -> Vec<String> {
        catalogue::search(&self.icons, &self.query).into_iter()
            .filter(|icon| match self.view {
                LibraryView::All => true,
                LibraryView::Favorites => self.settings.is_favorite(&icon.name),
                LibraryView::Recent => self.settings.recent.iter().any(|name| name == &icon.name),
            })
            .filter(|icon| self.group.as_ref().map_or(true, |group| {
                icon.tags.iter().any(|tag| tag == &format!("$group:{group}"))
            }))
            .map(|icon| icon.name.clone()).collect()
    }

    fn available_groups(&self) -> Vec<String> {
        let mut groups: Vec<String> = self.icons.iter().flat_map(|icon| icon.tags.iter())
            .filter_map(|tag| tag.strip_prefix("$group:").map(str::to_owned)).collect();
        groups.sort();
        groups.dedup();
        groups
    }

    fn export_svg(&mut self, name: &str, svg: &str) {
        let Some(path) = rfd::FileDialog::new().set_file_name(format!("{name}.svg"))
            .add_filter("SVG image", &["svg"]).save_file() else { return; };
        self.status = match fs::write(&path, svg) {
            Ok(()) => format!("Saved {}", path.display()),
            Err(error) => format!("Could not save SVG: {error}"),
        };
    }

    fn launch_figcraft(&mut self) {
        self.status = match FigCraftClient::new(self.settings.control_port).launch() {
            Ok(()) => format!("Launching FigCraft with control enabled on port {}…", self.settings.control_port),
            Err(error) => error.to_string(),
        };
    }

    fn check_figcraft(&mut self, ctx: &egui::Context) {
        let port = self.settings.control_port;
        let sender = self.action_tx.clone();
        let repaint = ctx.clone();
        self.status = format!("Checking FigCraft control connection on port {port}…");
        thread::spawn(move || {
            let result = FigCraftClient::new(port).check_connection()
                .map(|_| "FigCraft control connection is responding.".to_owned())
                .map_err(|error| error.to_string());
            let _ = sender.send(BackgroundResult::Connection(result));
            repaint.request_repaint();
        });
    }

    fn insert_into_figcraft(&mut self, icon: &Icon, ctx: &egui::Context) {
        if self.importing { return; }
        self.importing = true;
        let name = icon.name.clone();
        let svg = self.styled_svg(icon);
        let port = self.settings.control_port;
        let x = self.settings.import_x;
        let y = self.settings.import_y;
        let center = self.settings.center_on_view;
        let sender = self.action_tx.clone();
        let repaint = ctx.clone();
        thread::spawn(move || {
            let result = FigCraftClient::new(port)
                .insert_svg(&name, &svg, if center { None } else { Some(x) },
                    if center { None } else { Some(y) }, center)
                .map(|reply| format!("Inserted {name} into FigCraft. {reply}"))
                .map_err(|error| error.to_string());
            let _ = sender.send(BackgroundResult::Import(result));
            repaint.request_repaint();
        });
        self.status = format!("Importing {} into FigCraft…", icon.name);
    }

    fn render_preview(ctx: &egui::Context, svg: &str, key: &str) -> Option<egui::TextureHandle> {
        let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).ok()?;
        let source_size = tree.size().to_int_size();
        let scale = 4.0_f32;
        if !scale.is_finite() || scale <= 0.0 { return None; }
        let width = source_size.width().max(1) * scale as u32;
        let height = source_size.height().max(1) * scale as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
        resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [width as usize, height as usize], pixmap.data());
        Some(ctx.load_texture(key, image, egui::TextureOptions::LINEAR))
    }
}

impl eframe::App for CompanionApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(result) = self.catalogue_rx.try_recv() {
            self.loading = false;
            match result {
                Ok((icons, version)) => {
                    self.icons = icons;
                    self.selected = None;
                    self.preview = None;
                    self.grid_previews.clear();
                    self.status = format!("Lucide {version} · {} icons indexed with tags", self.icons.len());
                }
                Err(error) => self.status = format!("Catalogue update failed: {error}"),
            }
        }
        if let Ok(action) = self.action_rx.try_recv() {
            match action {
                BackgroundResult::Connection(result) => {
                    self.status = result.unwrap_or_else(|error| error);
                }
                BackgroundResult::Import(result) => {
                    self.importing = false;
                    self.status = result.unwrap_or_else(|error| error);
                }
            }
        }

    }

    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root_ui.ctx().clone();

        egui::Panel::top("header").show(root_ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Lucide");
                ui.label(egui::RichText::new("for FigCraft").weak().small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add_enabled(!self.loading, egui::Button::new("↻")).on_hover_text("Update icon library").clicked() {
                        self.update_catalogue(&ctx);
                    }
                    if ui.button("Test").clicked() { self.check_figcraft(&ctx); }
                    if ui.button("Open FigCraft").clicked() { self.launch_figcraft(); }
                });
            });
        });

        egui::Panel::bottom("status").show(root_ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status).small().weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("{} icons", self.icons.len())).small().weak());
                });
            });
        });

        egui::CentralPanel::default().show(root_ui, |ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query)
                .hint_text("Search icons…").desired_width(f32::INFINITY));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.view, LibraryView::All, "All");
                ui.selectable_value(&mut self.view, LibraryView::Favorites, "★");
                ui.selectable_value(&mut self.view, LibraryView::Recent, "Recent");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let groups = self.available_groups();
                    egui::ComboBox::from_id_salt("tag_group_filter")
                        .selected_text(self.group.as_deref().unwrap_or("Groups"))
                        .width(112.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.group, None, "All groups");
                            for group in &groups {
                                ui.selectable_value(&mut self.group, Some(group.clone()), group);
                            }
                        });
                });
            });

            let names = self.filtered_icon_names();
            ui.label(egui::RichText::new(format!("{} icons", names.len())).small().weak());

            if let Some(icon) = self.selected_icon() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&icon.name).strong());
                    if ui.add_enabled(!self.importing, egui::Button::new(if self.importing { "Inserting…" } else { "Insert ↗" })).clicked() {
                        self.insert_into_figcraft(&icon, &ctx);
                    }
                    if ui.small_button("Copy").on_hover_text("Copy SVG").clicked() {
                        ctx.copy_text(self.styled_svg(&icon));
                        self.status = format!("Copied {} SVG.", icon.name);
                    }
                    if ui.small_button(if self.settings.is_favorite(&icon.name) { "★" } else { "☆" }).clicked() {
                        self.settings.toggle_favorite(&icon.name);
                        let _ = self.settings.save();
                    }
                });
                ui.collapsing("Customize", |ui| {
                    ui.add(egui::Slider::new(&mut self.settings.stroke_width, 0.5..=4.0).text("Stroke"));
                    ui.horizontal(|ui| {
                        ui.label("Colour");
                        ui.add(egui::TextEdit::singleline(&mut self.settings.stroke_color).desired_width(82.0));
                        ui.label("Fill");
                        ui.add(egui::TextEdit::singleline(&mut self.settings.fill_color).desired_width(70.0));
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.settings.center_on_view, "Center in viewport");
                        if ui.button("Export SVG…").clicked() {
                            let svg = self.styled_svg(&icon);
                            self.export_svg(&icon.name, &svg);
                        }
                    });
                });
            } else {
                ui.label(egui::RichText::new("Select an icon · double-click to insert").small().weak());
            }

            ui.separator();

            let available_width = if ui.available_width().is_finite() {
                ui.available_width().clamp(1.0, 4096.0)
            } else { 360.0 };
            let cell_width = ((available_width - 16.0) / 3.0).clamp(88.0, 124.0);
            let columns = ((available_width / cell_width).floor() as usize).max(1);
            let row_height = 76.0_f32;
            let row_count = names.len().div_ceil(columns);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(
                ui, row_height, row_count, |ui, row_range| {
                    for row in row_range {
                        ui.horizontal(|ui| {
                            for column in 0..columns {
                                let index = row * columns + column;
                                let Some(name) = names.get(index) else { break; };
                                let selected = self.selected.as_ref() == Some(name);
                                let icon_svg = self.icons.iter()
                                    .find(|icon| icon.name == *name)
                                    .map(|icon| icon.svg.replace("currentColor", "#e5e7eb"));
                                ui.allocate_ui_with_layout(
                                    egui::vec2(cell_width, row_height),
                                    egui::Layout::top_down(egui::Align::Center),
                                    |ui| {
                                        if !self.grid_previews.contains_key(name) {
                                            if let Some(svg) = icon_svg.as_deref() {
                                                if let Some(texture) = Self::render_preview(&ctx, svg, &format!("grid-{name}")) {
                                                    self.grid_previews.insert(name.clone(), texture);
                                                }
                                            }
                                        }
                                        if let Some(texture) = self.grid_previews.get(name) {
                                            ui.add(egui::Image::new((texture.id(), egui::vec2(30.0, 30.0))));
                                        } else {
                                            ui.add_space(30.0);
                                        }
                                        let response = ui.add_sized(
                                            [cell_width - 4.0, 22.0],
                                            egui::Button::selectable(selected, name),
                                        );
                                        if response.double_clicked() {
                                            self.select_icon(name);
                                            if let Some(icon) = self.selected_icon() {
                                                self.insert_into_figcraft(&icon, &ctx);
                                            }
                                        } else if response.clicked() {
                                            self.select_icon(name);
                                        }
                                    },
                                );
                            }
                        });
                    }
                },
            );
        });
    }
}


#[cfg(test)]
mod tests {
    use super::safe_hex_color;

    #[test]
    fn accepts_supported_hex_colour_lengths() {
        assert_eq!(safe_hex_color("#abc"), Some("#abc"));
        assert_eq!(safe_hex_color("#aabbcc"), Some("#aabbcc"));
        assert_eq!(safe_hex_color("#aabbccdd"), Some("#aabbccdd"));
    }

    #[test]
    fn rejects_invalid_or_injected_colour_values() {
        assert_eq!(safe_hex_color("red"), None);
        assert_eq!(safe_hex_color("#12xz56"), None);
        assert_eq!(safe_hex_color("#fff\" onload=\"bad"), None);
        assert_eq!(safe_hex_color("none"), None);
    }
}
