use crate::{catalogue::{self, Icon}, figcraft::FigCraftClient, settings::Settings};
use eframe::egui;
use std::{fs, sync::mpsc::{self, Receiver, Sender}, thread};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryView { All, Favorites, Recent }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MainTab { Icons, Settings, Info }

enum BackgroundResult {
    Connection(Result<String, String>),
    Import(Result<String, String>),
    Context(Result<(String, Vec<u8>), String>),
}

pub struct CompanionApp {
    settings: Settings,
    query: String,
    icons: Vec<Icon>,
    catalogue_version: String,
    selected: Option<String>,
    view: LibraryView,
    group: Option<String>,
    category_open: bool,
    category_popup_pos: egui::Pos2,
    tab: MainTab,
    status: String,
    loading: bool,
    importing: bool,
    catalogue_tx: Sender<Result<(Vec<Icon>, String), String>>,
    catalogue_rx: Receiver<Result<(Vec<Icon>, String), String>>,
    action_tx: Sender<BackgroundResult>,
    action_rx: Receiver<BackgroundResult>,
    preview: Option<(String, egui::TextureHandle)>,
    grid_previews: std::collections::HashMap<String, egui::TextureHandle>,
    context_loading: bool,
    host_context: Option<(String, egui::TextureHandle)>,
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Match the Lucide plugin's restrained charcoal surface across every panel and popup.
        let mut visuals = egui::Visuals::dark();
        visuals.dark_mode = true;
        visuals.panel_fill = egui::Color32::from_rgb(43, 43, 43);
        visuals.window_fill = egui::Color32::from_rgb(43, 43, 43);
        visuals.extreme_bg_color = egui::Color32::from_rgb(32, 32, 32);
        visuals.faint_bg_color = egui::Color32::from_rgb(48, 48, 48);
        visuals.code_bg_color = egui::Color32::from_rgb(35, 35, 35);
        visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(68, 68, 68));
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(43, 43, 43);
        // Controls that sit on the toolbar should visually disappear into the window surface.
        visuals.widgets.inactive.bg_fill = visuals.panel_fill;
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(62, 62, 62);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(72, 72, 72);
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 105, 145);
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(62, 62, 62));
        cc.egui_ctx.set_visuals(visuals);
        let settings = Settings::load();
        let icons = catalogue::load_cache(&settings.cache_dir);
        let catalogue_version = fs::read_to_string(settings.cache_dir.join("version.txt"))
            .map(|version| version.trim().to_owned())
            .unwrap_or_else(|_| "version unavailable".to_owned());
        let status = if icons.is_empty() {
            "Catalogue not installed. Choose Update catalogue to download all Lucide icons.".to_owned()
        } else {
            format!("Ready · {} cached icons", icons.len())
        };
        let (catalogue_tx, catalogue_rx) = mpsc::channel();
        let (action_tx, action_rx) = mpsc::channel();
        Self {
            settings, query: String::new(), icons, catalogue_version, selected: None, view: LibraryView::All,
            group: None, category_open: false, category_popup_pos: egui::Pos2::ZERO,
            tab: MainTab::Icons, status, loading: false, importing: false,
            catalogue_tx, catalogue_rx, action_tx, action_rx, preview: None,
            grid_previews: std::collections::HashMap::new(),
            context_loading: false, host_context: None,
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
                let key = group.to_lowercase().replace(" & ", " ").replace('-', "").replace(' ', "");
                let metadata_match = icon.tags.iter().any(|tag| {
                    tag.strip_prefix("$group:").map(|value| value.eq_ignore_ascii_case(group)).unwrap_or(false)
                });
                if metadata_match { return true; }
                let haystack = format!("{} {}", icon.name.replace('-', " "), icon.tags.join(" ")).to_lowercase();
                let terms: Vec<String> = group.to_lowercase().split(|c: char| !c.is_alphanumeric())
                    .filter(|part| !part.is_empty()).map(str::to_owned).collect();
                terms.iter().any(|term| haystack.contains(term))
                    || haystack.contains(&key)
            }))
            .map(|icon| icon.name.clone()).collect()
    }

    fn available_groups(&self) -> Vec<String> {
        let mut groups: Vec<String> = self.icons.iter().flat_map(|icon| icon.tags.iter())
            .filter_map(|tag| tag.strip_prefix("$group:").map(str::to_owned)).collect();
        if groups.is_empty() {
            // lucide-static ships searchable tags but some releases omit category
            // group markers; keep the official category picker populated in that case.
            groups = [
                "Accessibility", "Accounts & access", "Animals", "Arrows", "Buildings",
                "Charts", "Communication", "Connectivity", "Cursors", "Design",
                "Coding & development", "Devices", "Emoji", "File icons", "Finance",
                "Food & beverage", "Gaming", "Home", "Layout", "Letters & digits",
                "Mail", "Mathematics", "Medical", "Multimedia", "Nature",
                "Navigation & places", "Notifications", "People", "Photography",
                "Science", "Seasons", "Security", "Shapes", "Shopping", "Social",
                "Sports", "Sustainability", "Time", "Travel", "Weather"
            ].iter().map(|value| value.to_string()).collect();
        }
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

    fn refresh_figcraft_context(&mut self, ctx: &egui::Context) {
        if self.context_loading { return; }
        self.context_loading = true;
        self.status = "Reading active FigCraft document and rendering its selection…".into();
        let port = self.settings.control_port;
        let sender = self.action_tx.clone();
        let repaint = ctx.clone();
        thread::spawn(move || {
            let result = FigCraftClient::new(port).inspect_and_render_selection()
                .map(|(document, png)| {
                    let details = serde_json::to_string_pretty(&document).unwrap_or_else(|_| document.to_string());
                    (details, png)
                })
                .map_err(|error| error.to_string());
            let _ = sender.send(BackgroundResult::Context(result));
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
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(result) = self.catalogue_rx.try_recv() {
            self.loading = false;
            match result {
                Ok((icons, version)) => {
                    self.icons = icons;
                    self.catalogue_version = version.clone();
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
                BackgroundResult::Context(result) => {
                    self.context_loading = false;
                    match result {
                        Ok((details, png)) => {
                            match image::load_from_memory(&png) {
                                Ok(image) => {
                                    let rgba = image.to_rgba8();
                                    let size = [rgba.width() as usize, rgba.height() as usize];
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                                    let texture = ctx.load_texture("figcraft-selection", color_image, egui::TextureOptions::LINEAR);
                                    self.host_context = Some((details, texture));
                                    self.status = "FigCraft document and selection refreshed.".into();
                                }
                                Err(error) => self.status = format!("Could not decode FigCraft selection preview: {error}"),
                            }
                        }
                        Err(error) => self.status = error,
                    }
                }
            }
        }

    }

    fn ui(&mut self, root_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root_ui.ctx().clone();
        // One compact navigation row: keep the window chrome, remove the redundant in-app title bar.
        egui::Panel::top("navigation")
            .frame(egui::Frame::new()
                .fill(root_ui.visuals().panel_fill)
                .stroke(egui::Stroke::NONE))
            .show(root_ui, |ui| {
            // Center the title across the window and vertically within the native titlebar row.
            #[cfg(target_os = "macos")]
            {
                let title_row_width = ui.available_width();
                let title_row = ui.allocate_ui_with_layout(
                    egui::vec2(title_row_width, 30.0),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label(egui::RichText::new("Lucid Icon")
                            .font(egui::FontId::proportional(16.0))
                            .strong()
                            .color(egui::Color32::from_gray(242)));
                    },
                );
                // Align native AppKit controls vertically with the title row.
                _frame.set_traffic_lights_position(&ctx, title_row.response.rect.y_range(), 12.0);
            }
            // Keep a clear visual gap between the title row and navigation tabs.
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.spacing_mut().item_spacing.x = 18.0;
                let tab_color = egui::Color32::from_gray(190);
                let active_color = egui::Color32::from_gray(248);
                if ui.add(egui::Button::new(egui::RichText::new("Icon").size(14.0)
                    .color(if self.tab == MainTab::Icons { active_color } else { tab_color })).frame(false)).clicked() {
                    self.tab = MainTab::Icons;
                }
                if ui.add(egui::Button::new(egui::RichText::new("Settings").size(14.0)
                    .color(if self.tab == MainTab::Settings { active_color } else { tab_color })).frame(false)).clicked() {
                    self.tab = MainTab::Settings;
                }
                if ui.add(egui::Button::new(egui::RichText::new("Info").size(14.0)
                    .color(if self.tab == MainTab::Info { active_color } else { tab_color })).frame(false)).clicked() {
                    self.tab = MainTab::Info;
                }
            });
            // Fixed navigation row height keeps the toolbar position predictable.
            ui.add_space(7.0);
        });

        egui::Panel::bottom("catalogue_footer")
            .frame(egui::Frame::new()
                .fill(root_ui.visuals().panel_fill)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::symmetric(12, 5)))
            .show_inside(root_ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Lucide v{}", self.catalogue_version))
                        .size(11.0).color(egui::Color32::from_gray(155)));
                    ui.add_space(4.0);
                    let refresh = ui.add_enabled(!self.loading, egui::Button::new(
                        egui::RichText::new("⟳").size(17.0).color(egui::Color32::from_gray(175))
                    ).frame(false));
                    if refresh.on_hover_text("Update Lucide catalogue").clicked() {
                        self.update_catalogue(&ctx);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let open = ui.add(egui::Button::new(
                            egui::RichText::new("Open FigCraft").size(10.5)
                                .color(egui::Color32::from_gray(205))
                        ).fill(egui::Color32::from_rgb(66, 66, 66))
                            .stroke(egui::Stroke::new(0.5, egui::Color32::from_gray(88))));
                        if open.on_hover_text("Open FigCraft with control enabled").clicked() {
                            self.launch_figcraft();
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::new()
                .fill(root_ui.visuals().panel_fill)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::symmetric(16, 0)))
            .show(root_ui, |ui| {
            match self.tab {
                MainTab::Icons => {
                    ui.add_space(3.0);
                    // Compute category groups outside the toolbar closure so the popup can use them.
                    let groups = self.available_groups();
                    // One aligned search row; category control is a text-only custom popup.
                    ui.horizontal(|ui| {
                        let gap = 10.0_f32;
                        let available = ui.available_width();
                        let selected_filter = match self.view {
                            LibraryView::Favorites => "Favorites",
                            LibraryView::Recent => "Recent",
                            LibraryView::All => self.group.as_deref().unwrap_or("All categories"),
                        };
                        // Measure the selected label and grow the category control to the left.
                        // Its right edge stays fixed, preserving the same gap before the chevron
                        // and the window's right content margin for both short and long labels.
                        let label_font = egui::FontId::proportional(13.0);
                        let label_galley = ui.painter().layout_no_wrap(
                            selected_filter.to_owned(),
                            label_font,
                            egui::Color32::from_gray(175),
                        );
                        let max_category_width = (available - gap - 112.0).max(146.0);
                        let category_width = (label_galley.size().x + 42.0)
                            .max(146.0)
                            .min(max_category_width);
                        let search_width = (available - category_width - gap).max(112.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(search_width, 30.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing.x = 7.0;
                                // Draw the magnifier with vector primitives so it renders on every system font.
                                let (search_icon_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(16.0, 30.0), egui::Sense::hover());
                                let search_center = egui::pos2(
                                    search_icon_rect.center().x - 2.0,
                                    search_icon_rect.center().y - 1.0,
                                );
                                let search_stroke = egui::Stroke::new(1.8, egui::Color32::WHITE);
                                ui.painter().circle_stroke(search_center, 6.0, search_stroke);
                                ui.painter().line_segment(
                                    [egui::pos2(search_center.x + 4.5, search_center.y + 4.5),
                                     egui::pos2(search_center.x + 9.0, search_center.y + 9.0)],
                                    search_stroke,
                                );
                                // Keep a fixed trailing clear-button slot so typing never
                                // changes the toolbar width or pushes the category picker offscreen.
                                let clear_slot_width = 22.0_f32;
                                let text_width = (ui.available_width() - clear_slot_width - 4.0).max(40.0);
                                let _response = ui.add_sized(
                                    [text_width, 30.0],
                                    egui::TextEdit::singleline(&mut self.query)
                                        .font(egui::FontId::proportional(14.0))
                                        .hint_text("Search")
                                        .vertical_align(egui::Align::Center)
                                        .frame(egui::Frame::NONE),
                                );
                                let _clear_slot = ui.allocate_ui_with_layout(
                                    egui::vec2(clear_slot_width, 30.0),
                                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                    |ui| {
                                        if self.query.is_empty() {
                                            ui.add_space(clear_slot_width);
                                        } else if ui.small_button("×").clicked() {
                                            self.query.clear();
                                        }
                                    },
                                );

                            },
                        );
                        ui.add_space(gap);
                        let category_button = ui.add_sized(
                            [category_width, 30.0],
                            egui::Button::new(
                                egui::RichText::new(selected_filter)
                                    .size(13.0)
                                    .color(egui::Color32::from_gray(175))
                            ).frame(false)
                        );
                        // Draw a true vector chevron; avoid font-dependent ▾ glyph fallback.
                        let chevron_center = egui::pos2(category_button.rect.right() - 15.0, category_button.rect.center().y);
                        let chevron_stroke = egui::Stroke::new(1.2, egui::Color32::from_gray(155));
                        ui.painter().line_segment(
                            [chevron_center + egui::vec2(-3.0, -1.5), chevron_center + egui::vec2(0.0, 1.5)],
                            chevron_stroke,
                        );
                        ui.painter().line_segment(
                            [chevron_center + egui::vec2(0.0, 1.5), chevron_center + egui::vec2(3.0, -1.5)],
                            chevron_stroke,
                        );
                        const CATEGORY_POPUP_WIDTH: f32 = 232.0;
                        self.category_popup_pos = egui::pos2(
                            category_button.rect.right() - CATEGORY_POPUP_WIDTH,
                            category_button.rect.bottom() + 4.0,
                        );
                        if category_button.clicked() {
                            self.category_open = !self.category_open;
                        }
                    });
                    if self.category_open {
                        let popup_pos = self.category_popup_pos;
                        let mut close_popup = false;
                        egui::Area::new(egui::Id::new("category_filter_popup"))
                            .order(egui::Order::Foreground)
                            .fixed_pos(popup_pos)
                            .show(&ctx, |ui| {
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    ui.set_min_width(232.0);
                                    ui.set_max_width(232.0);
                                    // Keep the list in a left-to-right child layout so egui places
                                    // the vertical scrollbar on the right edge, beside the rows.
                                    let mut popup_style = (*ui.style()).clone();
                                    popup_style.spacing.scroll.floating = false;
                                    ui.set_style(popup_style);
                                    if ui.selectable_label(self.view == LibraryView::All && self.group.is_none(), "All categories").clicked() {
                                        self.view = LibraryView::All;
                                        self.group = None;
                                        close_popup = true;
                                    }
                                    if ui.selectable_label(self.view == LibraryView::Favorites, "Favorites").clicked() {
                                        self.view = LibraryView::Favorites;
                                        close_popup = true;
                                    }
                                    if ui.selectable_label(self.view == LibraryView::Recent, "Recent").clicked() {
                                        self.view = LibraryView::Recent;
                                        close_popup = true;
                                    }
                                    ui.separator();
                                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                                        egui::ScrollArea::vertical()
                                            .max_height(420.0)
                                            .auto_shrink([false, true])
                                            .show(ui, |ui| {
                                                ui.set_min_width(208.0);
                                                for group in &groups {
                                                    if ui.selectable_label(
                                                        self.view == LibraryView::All && self.group.as_ref() == Some(group),
                                                        group,
                                                    ).clicked() {
                                                        self.view = LibraryView::All;
                                                        self.group = Some(group.clone());
                                                        close_popup = true;
                                                    }
                                                }
                                            });
                                    });
                                });
                            });
                        if close_popup {
                            self.category_open = false;
                        }
                    }
                    ui.add_space(0.0);
                    let names = self.filtered_icon_names();
                    // Recalculate the number and width of cells from the live viewport width.
                    // This keeps the icon grid centred and evenly spaced as the window is resized.
                    let grid_width = ui.available_width().max(1.0);
                    let columns = ((grid_width / 50.0).floor() as usize).clamp(5, 6);
                    let cell_width = grid_width / columns as f32;
                    let row_height = 58.0;
                    let rows = names.len().div_ceil(columns);
                    egui::ScrollArea::vertical().auto_shrink([false, false])
                        .show_rows(ui, row_height, rows, |ui, range| {
                            for row in range {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 0.0;
                                    for col in 0..columns {
                                        let index = row * columns + col;
                                        let Some(name) = names.get(index) else { break; };
                                        let selected = self.selected.as_ref() == Some(name);
                                        let svg = self.icons.iter().find(|icon| icon.name == *name)
                                            .map(|icon| icon.svg.replace("currentColor", "#eeeeee"));
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(cell_width, row_height),
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                let (rect, response) = ui.allocate_exact_size(
                                                    egui::vec2(54.0, 54.0), egui::Sense::click());
                                                if selected {
                                                    ui.painter().rect_filled(rect, 6.0, ui.visuals().selection.bg_fill);
                                                } else if response.hovered() {
                                                    ui.painter().rect_filled(rect, 6.0, ui.visuals().widgets.hovered.bg_fill);
                                                }
                                                if !self.grid_previews.contains_key(name) {
                                                    if let Some(svg) = svg.as_deref() {
                                                        if let Some(texture) = Self::render_preview(&ctx, svg, &format!("grid-{name}")) {
                                                            self.grid_previews.insert(name.clone(), texture);
                                                        }
                                                    }
                                                }
                                                if let Some(texture) = self.grid_previews.get(name) {
                                                    let image_rect = egui::Rect::from_center_size(
                                                        rect.center(), egui::vec2(32.0, 32.0));
                                                    ui.painter().image(texture.id(), image_rect,
                                                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                                                        egui::Color32::WHITE);
                                                }
                                                if response.on_hover_text(name).clicked() {
                                                    self.select_icon(name);
                                                    if let Some(icon) = self.selected_icon() {
                                                        self.insert_into_figcraft(&icon, &ctx);
                                                    }
                                                }
                                            });
                                    }
                                });
                            }
                        });
                    if let Some(icon) = self.selected_icon() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&icon.name).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add_enabled(!self.importing, egui::Button::new(
                                    if self.importing { "Inserting…" } else { "Insert" })).clicked() {
                                    self.insert_into_figcraft(&icon, &ctx);
                                }
                                if ui.small_button("Copy").clicked() {
                                    ctx.copy_text(self.styled_svg(&icon));
                                    self.status = format!("Copied {} SVG.", icon.name);
                                }
                                if ui.small_button(if self.settings.is_favorite(&icon.name) { "★" } else { "☆" }).clicked() {
                                    self.settings.toggle_favorite(&icon.name);
                                    let _ = self.settings.save();
                                }
                            });
                        });
                        ui.collapsing("Quick customize", |ui| {
                            let mut changed = false;
                            ui.horizontal(|ui| {
                                changed |= ui.add(egui::Slider::new(&mut self.settings.stroke_width, 0.5..=4.0)
                                    .text("Stroke")).changed();
                                changed |= ui.add(egui::Slider::new(&mut self.settings.icon_size, 48.0..=240.0)
                                    .text("Size")).changed();
                            });
                            if changed { self.save_settings(); }
                            if ui.button("Export styled SVG…").clicked() {
                                let svg = self.styled_svg(&icon);
                                self.export_svg(&icon.name, &svg);
                            }
                        });
                    }
                }
                MainTab::Settings => {
                    ui.add_space(10.0);
                    ui.columns(2, |columns| {
                        columns[0].label(egui::RichText::new("Icon size").strong());
                        columns[0].add(egui::Slider::new(&mut self.settings.icon_size, 16.0..=240.0)
                            .show_value(true).suffix(" px"));
                        columns[1].label(egui::RichText::new("Stroke width").strong());
                        columns[1].add(egui::Slider::new(&mut self.settings.stroke_width, 0.5..=4.0)
                            .show_value(true));
                    });
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Stroke colour");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.settings.stroke_color).desired_width(110.0));
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Fill colour");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.settings.fill_color).desired_width(110.0));
                        });
                    });
                    let valid = safe_hex_color(&self.settings.stroke_color).is_some()
                        && (self.settings.fill_color.trim().eq_ignore_ascii_case("none")
                            || safe_hex_color(&self.settings.fill_color).is_some());
                    if !valid {
                        ui.colored_label(egui::Color32::LIGHT_RED, "Use a valid hex colour or 'none'.");
                    }
                    ui.separator();
                    ui.checkbox(&mut self.settings.center_on_view, "Center inserted icons in viewport");
                    if !self.settings.center_on_view {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.settings.import_x).prefix("X "));
                            ui.add(egui::DragValue::new(&mut self.settings.import_y).prefix("Y "));
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label("FigCraft control port");
                        ui.add(egui::DragValue::new(&mut self.settings.control_port).range(1..=65535));
                    });
                    if ui.button("Save settings").clicked() && valid { self.save_settings(); }
                    ui.add_space(8.0);
                    if ui.button("Reset settings").clicked() {
                        self.settings = Settings::default();
                        self.save_settings();
                    }
                }
                MainTab::Info => {
                    ui.add_space(12.0);
                    ui.heading("Lucide Companion");
                    ui.label("A native icon browser and SVG importer for FigCraft.");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("BUILD INFORMATION").small().strong().weak());
                    ui.label(format!("App version · {}", env!("CARGO_PKG_VERSION")));
                    ui.label(format!("Build type · {}", option_env!("LUCIDE_BUILD_CHANNEL").unwrap_or("development")));
                    ui.label(format!("Branch · {}", option_env!("LUCIDE_BUILD_BRANCH").unwrap_or("unknown")));
                    ui.label(format!("Commit · {}", option_env!("LUCIDE_BUILD_COMMIT").unwrap_or("unknown")));
                    ui.label(format!("Built · {}", option_env!("LUCIDE_BUILD_DATE").unwrap_or("unknown")));
                    ui.add_space(6.0);
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(&self.status).small().weak());
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("FIGCRAFT CONNECTION").small().strong().weak());
                    ui.label(format!("Control endpoint · 127.0.0.1:{}", self.settings.control_port));
                    ui.horizontal(|ui| {
                        if ui.button("Open FigCraft").clicked() { self.launch_figcraft(); }
                        if ui.button("Test connection").clicked() { self.check_figcraft(&ctx); }
                    });
                    if ui.button("Inspect active document").clicked() {
                        self.refresh_figcraft_context(&ctx);
                    }
                    if let Some((details, texture)) = &self.host_context {
                        ui.separator();
                        ui.label("Active selection preview");
                        let size = texture.size_vec2();
                        let scale = (ui.available_width() / size.x).min(1.0);
                        ui.image((texture.id(), size * scale));
                        ui.collapsing("Document details (JSON)", |ui| {
                            egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                                ui.code(details);
                            });
                        });
                    }
                    ui.separator();
                    ui.label("Lucide SVG icon library");
                    ui.label(format!("{} icons cached locally", self.icons.len()));
                    ui.hyperlink_to("Lucide project", "https://lucide.dev/");
                    ui.hyperlink_to("Source repository", "https://github.com/dsvictor/Lucid-Icon-Companion");
                }
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
