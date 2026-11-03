use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub control_port: u16,
    pub icon_size: f32,
    pub stroke_width: f32,
    pub stroke_color: String,
    pub fill_color: String,
    pub catalogue_cache: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        let base = directories::ProjectDirs::from("dev", "Jayanta", "LucideCompanion")
            .map(|d| d.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".lucide-companion"));
        Self {
            control_port: 7979,
            icon_size: 24.0,
            stroke_width: 2.0,
            stroke_color: "#000000".into(),
            fill_color: "none".into(),
            catalogue_cache: base.join("icons"),
        }
    }
}

impl Settings {
    pub fn load() -> Self { Self::default() }
}
