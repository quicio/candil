//! Preferencias y progreso en disco (JSON, sin base de datos).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::store;
use crate::theme::THEMES;

/// Rangos únicos de tipografía: los usan el clamp y los sliders de la UI.
pub const FONT_MIN: f32 = 12.0;
pub const FONT_MAX: f32 = 34.0;
pub const LINE_MIN: f32 = 1.15;
pub const LINE_MAX: f32 = 2.3;
pub const COLUMN_MIN: f32 = 400.0;
pub const COLUMN_MAX: f32 = 1200.0;
pub const PARAGRAPH_MIN: f32 = 0.0;
pub const PARAGRAPH_MAX: f32 = 1.6;

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
        self.font_size = self.font_size.clamp(FONT_MIN, FONT_MAX);
        self.line_height = self.line_height.clamp(LINE_MIN, LINE_MAX);
        self.column_width = self.column_width.clamp(COLUMN_MIN, COLUMN_MAX);
        self.paragraph_space = self.paragraph_space.clamp(PARAGRAPH_MIN, PARAGRAPH_MAX);
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub chapter: usize,
    /// Fracción de scroll dentro del capítulo (0..1).
    pub ratio: f32,
    /// Índice del bloque visible superior; sobrevive cambios de fuente.
    pub anchor: Option<usize>,
    /// Compatibilidad con `{chapter, offset}` de M2; se usa una vez al abrir
    /// el libro y se descarta al volver a guardar.
    #[serde(alias = "offset", default, skip_serializing_if = "Option::is_none")]
    pub legacy_offset: Option<f32>,
}

impl Progress {
    pub fn is_empty(&self) -> bool {
        self.chapter == 0
            && self.ratio <= 0.0
            && self.anchor.is_none()
            && self.legacy_offset.is_none_or(|offset| offset <= 0.0)
    }

    pub fn has_started(&self) -> bool {
        !self.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub settings: Settings,
    pub last_book: Option<String>,
    pub recent: Vec<String>,
    /// Compatibilidad: el progreso se migra a `library.json` en M2. No se
    /// vuelve a escribir acá.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub progress: BTreeMap<String, Progress>,
}

impl Config {
    pub fn data_dir() -> PathBuf {
        store::data_dir()
    }

    pub fn file() -> PathBuf {
        Self::data_dir().join("config.json")
    }

    pub fn load() -> Self {
        Self::load_with_report().0
    }

    /// Carga y, si el archivo estaba corrupto, devuelve un mensaje para la UI.
    pub fn load_with_report() -> (Self, Option<String>) {
        Self::load_with_report_from(&Self::file())
    }

    pub fn load_from(path: &Path) -> Self {
        Self::load_with_report_from(path).0
    }

    pub fn load_with_report_from(path: &Path) -> (Self, Option<String>) {
        let (mut cfg, report) = store::load_json_with_report::<Self>(path, "configuración");
        cfg.settings.clamp();
        (cfg, report)
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::file())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        store::save_json_atomic(path, self)
    }

    pub fn touch_recent(&mut self, key: &str) {
        self.recent.retain(|r| r != key);
        self.recent.insert(0, key.to_string());
        self.recent.truncate(12);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("lector-cfg-{tag}-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn roundtrip_sin_dejar_temporal() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("config.json");
        let mut cfg = Config::default();
        cfg.settings.font_size = 21.5;
        cfg.progress.insert(
            "/libro.epub".into(),
            Progress {
                chapter: 7,
                ratio: 0.5,
                anchor: Some(3),
                legacy_offset: None,
            },
        );
        cfg.save_to(&path).expect("guardar");

        assert!(!dir.join("config.json.tmp").exists());
        let loaded = Config::load_from(&path);
        assert!((loaded.settings.font_size - 21.5).abs() < f32::EPSILON);
        let progress = loaded.progress.get("/libro.epub").unwrap();
        assert_eq!(progress.chapter, 7);
        assert!((progress.ratio - 0.5).abs() < f32::EPSILON);
        assert_eq!(progress.anchor, Some(3));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn json_corrupto_hace_backup_y_arranca_con_default() {
        let dir = temp_dir("corrupt");
        let path = dir.join("config.json");
        std::fs::write(&path, "{ esto no es json").unwrap();

        let (cfg, report) = Config::load_with_report_from(&path);
        assert!(report.is_some(), "debe avisar del problema");
        assert_eq!(cfg.settings.font_size, Settings::default().font_size);
        assert!(!path.exists(), "el original se renombra");

        let backups: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".corrupt-"))
            .collect();
        assert_eq!(backups.len(), 1, "debe quedar una copia");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn json_viejo_sin_campos_nuevos_carga() {
        let dir = temp_dir("viejo");
        let path = dir.join("config.json");
        std::fs::write(&path, r#"{"settings":{"font_size":22.0}}"#).unwrap();

        let cfg = Config::load_from(&path);
        assert!((cfg.settings.font_size - 22.0).abs() < f32::EPSILON);
        assert_eq!(cfg.settings.theme, Settings::default().theme);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn settings_fuera_de_rango_se_clampan() {
        let dir = temp_dir("clamp");
        let path = dir.join("config.json");
        std::fs::write(
            &path,
            r#"{"settings":{"font_size":999.0,"line_height":-3.0,"column_width":1.0,"paragraph_space":9.0}}"#,
        )
        .unwrap();

        let cfg = Config::load_from(&path);
        assert_eq!(cfg.settings.font_size, FONT_MAX);
        assert_eq!(cfg.settings.line_height, LINE_MIN);
        assert_eq!(cfg.settings.column_width, COLUMN_MIN);
        assert_eq!(cfg.settings.paragraph_space, PARAGRAPH_MAX);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn progress_viejo_con_offset_se_migra_al_leer() {
        let old: Progress = serde_json::from_str(r#"{"chapter":3,"offset":42.5}"#).unwrap();
        assert_eq!(old.chapter, 3);
        assert_eq!(old.legacy_offset, Some(42.5));
        assert_eq!(old.ratio, 0.0);
        assert!(!old.is_empty());

        let new = Progress {
            chapter: 2,
            ratio: 0.25,
            anchor: Some(7),
            legacy_offset: None,
        };
        let json = serde_json::to_string(&new).unwrap();
        assert!(!json.contains("\"offset\""), "{json}");
        assert!(json.contains("\"ratio\""), "{json}");
        let back: Progress = serde_json::from_str(&json).unwrap();
        assert_eq!(back.anchor, Some(7));
        assert!((back.ratio - 0.25).abs() < f32::EPSILON);
    }
}
