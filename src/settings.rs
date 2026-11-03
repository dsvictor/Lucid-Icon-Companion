use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub control_port: u16,
    pub icon_size: f32,
    pub stroke_width: f32,
    pub stroke_color: String,
    pub fill_color: String,
    pub import_x: f64,
    pub import_y: f64,
    pub center_on_view: bool,
    pub cache_dir: PathBuf,
    pub favorites: Vec<String>,
    pub recent: Vec<String>,
    #[serde(skip)]
    settings_file: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        let data_dir = directories::ProjectDirs::from("dev", "Jayanta", "LucideIconCompanion")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".lucide-icon-companion"));

        Self {
            control_port: 7979,
            icon_size: 144.0,
            stroke_width: 2.0,
            stroke_color: "#000000".into(),
            fill_color: "none".into(),
            import_x: 0.0,
            import_y: 0.0,
            center_on_view: true,
            cache_dir: data_dir.join("icons"),
            favorites: Vec::new(),
            recent: Vec::new(),
            settings_file: data_dir.join("settings.json"),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let defaults = Self::default();
        let mut loaded = fs::read_to_string(&defaults.settings_file)
            .ok()
            .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
            .unwrap_or_else(|| defaults.clone());
        loaded.settings_file = defaults.settings_file;
        if loaded.cache_dir.as_os_str().is_empty() {
            loaded.cache_dir = defaults.cache_dir;
        }
        loaded
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.settings_file.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&self.settings_file, json).map_err(|e| e.to_string())
    }

    pub fn is_favorite(&self, name: &str) -> bool {
        self.favorites.iter().any(|item| item == name)
    }

    pub fn toggle_favorite(&mut self, name: &str) {
        if self.is_favorite(name) {
            self.favorites.retain(|item| item != name);
        } else {
            self.favorites.push(name.to_owned());
            self.favorites.sort();
        }
    }

    pub fn remember(&mut self, name: &str) {
        self.recent.retain(|item| item != name);
        self.recent.insert(0, name.to_owned());
        self.recent.truncate(40);
    }
}
