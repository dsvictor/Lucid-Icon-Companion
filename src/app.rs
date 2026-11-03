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
                ui.heading("Lucide Companion");
                ui.label("for FigCraft");
                ui.separator();
                if ui.add_enabled(!self.loading, egui::Button::new("Update catalogue")).clicked() {
                    self.update_catalogue(&ctx);
                }
                if ui.button("Launch FigCraft").clicked() { self.launch_figcraft(); }
                if ui.button("Test connection").clicked() { self.check_figcraft(&ctx); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("by Jayanta");
                });
            });
        });

        egui::Panel::bottom("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} icons", self.icons.len()));
                });
            });
        });

        egui::Panel::left("icon_browser").resizable(true).default_size(320.0).show(ctx, |ui| {
            ui.heading("Icon library");
            let search_response = ui.add(egui::TextEdit::singleline(&mut self.query)
                .hint_text("Search names and tags…").desired_width(f32::INFINITY));
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::F)) {
                search_response.request_focus();
            }
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.view, LibraryView::All, "All");
                ui.selectable_value(&mut self.view, LibraryView::Favorites, "★ Favorites");
                ui.selectable_value(&mut self.view, LibraryView::Recent, "Recent");
            });

            let groups = self.available_groups();
            egui::ComboBox::from_id_salt("tag_group_filter")
                .selected_text(self.group.as_deref().unwrap_or("All tag groups"))
                .width(f32::INFINITY)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.group, None, "All tag groups");
                    for group in &groups {
                        ui.selectable_value(&mut self.group, Some(group.clone()), group);
                    }
                });

            ui.separator();
            let names = self.filtered_icon_names();
            ui.label(format!("{} matching icons", names.len()));
            if !search_response.has_focus() {
                let (down, up, enter, escape) = ui.input(|i| (
                    i.key_pressed(egui::Key::ArrowDown),
                    i.key_pressed(egui::Key::ArrowUp),
                    i.key_pressed(egui::Key::Enter),
                    i.key_pressed(egui::Key::Escape),
                ));
                if escape && !self.query.is_empty() { self.query.clear(); }
                let current = self.selected.as_ref().and_then(|name| names.iter().position(|item| item == name));
                let next = if down && !names.is_empty() {
                    Some(current.map_or(0, |index| (index + 1).min(names.len() - 1)))
                } else if up && !names.is_empty() {
                    Some(current.map_or(0, |index| index.saturating_sub(1)))
                } else if enter && current.is_none() && !names.is_empty() {
                    Some(0)
                } else { None };
                if let Some(index) = next {
                    if let Some(name) = names.get(index) { self.select_icon(name); }
                }
            }

            // Virtualized thumbnail grid: only rows visible in the scroll viewport
            // are rasterized, keeping the full Lucide catalogue responsive.
            let available_width = if ui.available_width().is_finite() {
                ui.available_width().clamp(1.0, 4096.0)
            } else { 320.0 };
            let cell_width = 92.0_f32;
            let columns = ((available_width / cell_width).floor() as usize).max(1);
            let row_height = 88.0_f32;
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
                                    .map(|icon| icon.svg.clone());
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
                                            ui.add(egui::Image::new((texture.id(), egui::vec2(32.0, 32.0))));
                                        } else {
                                            ui.add_space(32.0);
                                        }
                                        if ui.add_sized(
                                            [cell_width - 4.0, 22.0],
                                            egui::Button::selectable(selected, name),
                                        ).clicked() {
                                            self.select_icon(name);
                                        }
                                        let favorite = self.settings.is_favorite(name);
                                        if ui.small_button(if favorite { "★" } else { "☆" }).clicked() {
                                            self.settings.toggle_favorite(name);
                                            let _ = self.settings.save();
                                        }
                                    },
                                );
                            }
                        });
                    }
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(icon) = self.selected_icon() {
                ui.horizontal(|ui| {
                    ui.heading(&icon.name);
                    if ui.button(if self.settings.is_favorite(&icon.name) { "★ Remove favorite" } else { "☆ Add favorite" }).clicked() {
                        self.settings.toggle_favorite(&icon.name);
                        let _ = self.settings.save();
                    }
                });
                let visible_tags: Vec<&str> = icon.tags.iter()
                    .filter(|tag| !tag.starts_with("$group:")).map(String::as_str).collect();
                let group_tags: Vec<&str> = icon.tags.iter()
                    .filter_map(|tag| tag.strip_prefix("$group:")).collect();
                ui.label(if visible_tags.is_empty() { "No descriptive tags supplied upstream".to_owned() } else { visible_tags.join(" · ") });
                if !group_tags.is_empty() { ui.label(format!("Groups: {}", group_tags.join(" · "))); }

                ui.separator();
                if !self.settings.icon_size.is_finite() {
                    self.settings.icon_size = 96.0;
                }
                self.settings.icon_size = self.settings.icon_size.clamp(24.0, 320.0);
                if !self.settings.stroke_width.is_finite() {
                    self.settings.stroke_width = 2.0;
                }
                self.settings.stroke_width = self.settings.stroke_width.clamp(0.5, 4.0);
                if !self.settings.import_x.is_finite() { self.settings.import_x = 0.0; }
                if !self.settings.import_y.is_finite() { self.settings.import_y = 0.0; }

                ui.collapsing("Appearance", |ui| {
                    ui.add(egui::Slider::new(&mut self.settings.icon_size, 24.0..=320.0).text("Preview size"));
                    ui.add(egui::Slider::new(&mut self.settings.stroke_width, 0.5..=4.0).text("Stroke width"));
                    ui.horizontal(|ui| {
                        ui.label("Stroke"); ui.text_edit_singleline(&mut self.settings.stroke_color);
                        ui.label("Fill"); ui.text_edit_singleline(&mut self.settings.fill_color);
                    });
                    ui.label("Use hex colours such as #171717, or set fill to none.");
                });

                let svg = self.styled_svg(&icon);
                let key = format!("{}-{}-{}-{}-{}", icon.name, self.settings.icon_size,
                    self.settings.stroke_width, self.settings.stroke_color, self.settings.fill_color);
                if self.preview.as_ref().map(|(cached, _)| cached) != Some(&key) {
                    self.preview = Self::render_preview(&ctx, &svg, &key).map(|texture| (key.clone(), texture));
                }
                ui.add_space(10.0);
                if let Some((_, texture)) = &self.preview {
                    ui.centered_and_justified(|ui| {
                        let preview_size = if self.settings.icon_size.is_finite() {
                            self.settings.icon_size.clamp(24.0, 320.0)
                        } else { 96.0 };
                        let size = egui::Vec2::splat(preview_size);
                        ui.add(egui::Image::new((texture.id(), size)));
                    });
                } else { ui.label("SVG preview could not be rendered."); }

                ui.separator();
                ui.heading("FigCraft insertion");
                ui.horizontal(|ui| {
                    ui.label("Control port");
                    ui.add(egui::DragValue::new(&mut self.settings.control_port).range(1..=65535));
                    ui.checkbox(&mut self.settings.center_on_view, "Center in viewport");
                });
                if !self.settings.center_on_view {
                    ui.horizontal(|ui| {
                        ui.label("Canvas X"); ui.add(egui::DragValue::new(&mut self.settings.import_x).speed(1.0));
                        ui.label("Y"); ui.add(egui::DragValue::new(&mut self.settings.import_y).speed(1.0));
                    });
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(!self.importing, egui::Button::new("Insert into FigCraft")).clicked() {
                        self.insert_into_figcraft(&icon, &ctx);
                    }
                    if ui.button("Export SVG…").clicked() { self.export_svg(&icon.name, &svg); }
                    if ui.button("Copy SVG").clicked() {
                        ctx.copy_text(svg.clone());
                        self.status = format!("Copied {} SVG to clipboard.", icon.name);
                    }
                    if ui.button("Save preferences").clicked() { self.save_settings(); }
                });
                ui.collapsing("SVG source", |ui| {
                    let mut source = svg.clone();
                    ui.add(egui::TextEdit::multiline(&mut source).desired_rows(10).code_editor().desired_width(f32::INFINITY));
                });
            } else {
                ui.centered_and_justified(|ui| { ui.label("Select an icon to preview, customize, export or insert it."); });
            }
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
