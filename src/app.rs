use crate::{catalogue::{self, Icon}, figcraft::FigCraftClient, settings::Settings};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct CompanionApp {
    settings: Settings,
    query: String,
    icons: Vec<Icon>,
    selected: Option<usize>,
    status: String,
    loading: bool,
    shared: Arc<Mutex<()>>,
}

impl CompanionApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            settings: Settings::load(),
            query: String::new(),
            icons: Vec::new(),
            selected: None,
            status: "Ready. Download the complete Lucide catalogue to begin.".into(),
            loading: false,
            shared: Arc::new(Mutex::new(())),
        }
    }
}

impl eframe::App for CompanionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Lucide Companion");
                ui.label("for FigCraft");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Download full Lucide catalogue").clicked() && !self.loading {
                        self.loading = true;
                        self.status = "Catalogue download must run asynchronously in the production build.".into();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} icons indexed", self.icons.len()));
                });
            });
        });

        egui::SidePanel::left("browser").resizable(true).default_width(290.0).show(ctx, |ui| {
            ui.heading("Icon library");
            ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search icons and tags…"));
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, icon) in catalogue::search(&self.icons, &self.query).into_iter().enumerate() {
                    let source_idx = self.icons.iter().position(|x| x.name == icon.name).unwrap_or(idx);
                    if ui.selectable_label(self.selected == Some(source_idx), &icon.name).clicked() {
                        self.selected = Some(source_idx);
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Preview & export");
            if let Some(idx) = self.selected {
                if let Some(icon) = self.icons.get(idx) {
                    ui.label(&icon.name);
                    ui.group(|ui| {
                        ui.label("SVG source preview");
                        ui.add(egui::TextEdit::multiline(&mut icon.svg.clone()).desired_rows(14).code_editor());
                    });
                    ui.horizontal(|ui| {
                        ui.add(egui::Slider::new(&mut self.settings.icon_size, 12.0..=256.0).text("Size"));
                        ui.add(egui::Slider::new(&mut self.settings.stroke_width, 0.5..=4.0).text("Stroke"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Stroke");
                        ui.text_edit_singleline(&mut self.settings.stroke_color);
                        ui.label("Fill");
                        ui.text_edit_singleline(&mut self.settings.fill_color);
                    });
                    if ui.button("Insert into FigCraft").clicked() {
                        let client = FigCraftClient::new(self.settings.control_port);
                        self.status = "FigCraft insertion protocol is not yet verified against FigCraft 0.15.0; insert is disabled until confirmed.".into();
                        let _ = client;
                    }
                    if ui.button("Save SVG…").clicked() {
                        self.status = "SVG save dialog is being added.".into();
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Choose an icon from the library to preview and export it.");
                });
            }
        });
        let _ = &self.shared;
    }
}
