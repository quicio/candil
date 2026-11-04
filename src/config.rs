//! Preferencias y progreso en disco (JSON, sin base de datos).

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::theme::THEMES;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: usize,
    pub font_size: f32,
    pub line_height: f32,
    pub column_width: f32,
    pub paragraph_space: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: 0,
            font_size: 19.0,
            line_height: 1.55,
            column_width: 660.0,
            paragraph_space: 0.75,
        }
    }
}

impl Settings {
    pub fn clamp(&mut self) {
        self.theme = self.theme.min(THEMES.len() - 1);
        self.font_size = self.font_size.clamp(12.0, 34.0);
        self.line_height = self.line_height.clamp(1.15, 2.3);
        self.column_width = self.column_width.clamp(400.0, 1200.0);
        self.paragraph_space = self.paragraph_space.clamp(0.0, 1.6);
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Progress {
    pub chapter: usize,
    pub offset: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub settings: Settings,
    pub last_book: Option<String>,
    pub recent: Vec<String>,
    pub progress: BTreeMap<String, Progress>,
}

impl Config {
    pub fn file() -> PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lector");
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let mut cfg: Self = std::fs::read_to_string(Self::file())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        cfg.settings.clamp();
        cfg
    }

    pub fn save(&self) {
        let path = Self::file();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn touch_recent(&mut self, key: &str) {
        self.recent.retain(|r| r != key);
        self.recent.insert(0, key.to_string());
        self.recent.truncate(12);
    }
}
