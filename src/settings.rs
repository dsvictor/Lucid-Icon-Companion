use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub control_port: u16,
    pub icon_size: f32,
    pub stroke_width: f32,
    pub stroke_color: String,
    pub fill_color: String,
    pub cache_dir: PathBuf,
    settings_file: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = directories::ProjectDirs::from("dev", "Jayanta", "LucideIconCompanion")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".lucide-icon-companion"));

        Self {
            control_port: 7979,
            icon_size: 128.0,
            stroke_width: 2.0,
            stroke_color: "#000000".into(),
            fill_color: "none".into(),
            cache_dir: data_dir.join("icons"),
            settings_file: data_dir.join("settings.json"),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let defaults = Self::default();
        fs::read_to_string(&defaults.settings_file)
            .ok()
            .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
            .unwrap_or(defaults)
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.settings_file.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&self.settings_file, json).map_err(|e| e.to_string())
    }
}
