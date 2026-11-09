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
    /// Fracción de scroll dentro del capítulo (0..1). Sirve para la tarjeta de
    /// biblioteca y como fallback cuando no hay ancla.
    pub ratio: f32,
    /// Índice del bloque visible superior; sobrevive cambios de fuente.
    pub anchor: Option<usize>,
    /// Fracción dentro de ese bloque (0..1). Sin esto, un párrafo largo puede
    /// descolocarse decenas de líneas al cambiar la tipografía.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_ratio: Option<f32>,
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
            && self.anchor_ratio.is_none_or(|ratio| ratio <= 0.0)
            && self.legacy_offset.is_none_or(|offset| offset <= 0.0)
    }

    pub fn has_started(&self) -> bool {
        !self.is_empty()
    }
}

pub const DEFAULT_AI_BASE: &str = "https://opencode.ai/inference/go/openai/v1";
pub const DEFAULT_AI_MODEL: &str = "deepseek-v4.1-flash";
pub const HISTORY_DEFAULT: usize = 200;
pub const AI_TIMEOUT_DEFAULT: u64 = 30;
pub const AI_MAX_TOKENS_DEFAULT: u32 = 512;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsultaSettings {
    /// La red está apagada hasta que el usuario la habilite.
    pub online_enabled: bool,
    pub dictionary_provider: String,
    pub translation_provider: String,
    /// Modelo de Ollama para traducción local; vacío = default del proveedor.
    #[serde(default)]
    pub translation_model: String,
    /// `None` = automático (en→es, es→en, resto→es).
    pub target_lang: Option<String>,
    pub ai_provider: String,
    pub ai_base_url: String,
    pub ai_model: String,
    pub ai_timeout_secs: u64,
    pub ai_max_tokens: u32,
    /// 0 = no guardar historial.
    pub history_limit: usize,
}

impl Default for ConsultaSettings {
    fn default() -> Self {
        Self {
            online_enabled: false,
            dictionary_provider: "wiktionary".into(),
            translation_provider: "mymemory".into(),
            translation_model: String::new(),
            target_lang: None,
            ai_provider: "opencode".into(),
            ai_base_url: DEFAULT_AI_BASE.into(),
            ai_model: DEFAULT_AI_MODEL.into(),
            ai_timeout_secs: AI_TIMEOUT_DEFAULT,
            ai_max_tokens: AI_MAX_TOKENS_DEFAULT,
            history_limit: HISTORY_DEFAULT,
        }
    }
}

impl ConsultaSettings {
    pub fn clamp(&mut self) {
        self.ai_timeout_secs = self.ai_timeout_secs.clamp(5, 120);
        self.ai_max_tokens = self.ai_max_tokens.clamp(64, 4096);
        self.history_limit = self.history_limit.min(2000);
        if !crate::ai::is_valid(&self.ai_provider) {
            self.ai_provider = "opencode".into();
        }
        let provider = crate::ai::provider(&self.ai_provider);
        if self.ai_base_url.trim().is_empty() {
            self.ai_base_url = provider.base_url.into();
        }
        if self.ai_model.trim().is_empty() {
            self.ai_model = provider.default_model.into();
        }
        if self.dictionary_provider.trim().is_empty() {
            self.dictionary_provider = "wiktionary".into();
        }
        if self.translation_provider.trim().is_empty() {
            self.translation_provider = "mymemory".into();
        }
        if self.translation_provider != "ollama" && self.translation_provider != "mymemory" {
            self.translation_provider = "mymemory".into();
        }
        if self.translation_provider == "ollama" && self.translation_model.trim().is_empty() {
            self.translation_model = crate::ai::provider("ollama").default_model.into();
        }
        if let Some(lang) = &self.target_lang {
            if lang.trim().is_empty() {
                self.target_lang = None;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupMode {
    /// Abrir el último libro si sigue existiendo; si no, la biblioteca.
    #[default]
    LastBook,
    /// Abrir siempre la biblioteca.
    Library,
}

impl StartupMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::LastBook => "Último libro abierto",
            Self::Library => "Biblioteca",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub settings: Settings,
    pub last_book: Option<String>,
    pub recent: Vec<String>,
    /// Qué abrir al iniciar la app.
    pub startup: StartupMode,
    pub consulta: ConsultaSettings,
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
        cfg.consulta.clamp();
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
            std::env::temp_dir().join(format!("candil-cfg-{tag}-{}-{stamp}", std::process::id()));
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
                anchor_ratio: Some(0.4),
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
        assert!((progress.anchor_ratio.unwrap() - 0.4).abs() < f32::EPSILON);
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
        assert_eq!(cfg.startup, StartupMode::LastBook);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn startup_se_puede_cambiar_a_biblioteca() {
        let cfg: Config = serde_json::from_str(r#"{"startup":"library"}"#).unwrap();
        assert_eq!(cfg.startup, StartupMode::Library);
        assert_eq!(StartupMode::Library.label(), "Biblioteca");
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
            anchor_ratio: None,
            legacy_offset: None,
        };
        let json = serde_json::to_string(&new).unwrap();
        assert!(!json.contains("\"offset\""), "{json}");
        assert!(json.contains("\"ratio\""), "{json}");
        let back: Progress = serde_json::from_str(&json).unwrap();
        assert_eq!(back.anchor, Some(7));
        assert!((back.ratio - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn consulta_apagada_por_defecto_y_roundtrip() {
        let mut cfg = Config::default();
        assert!(!cfg.consulta.online_enabled);
        assert_eq!(cfg.consulta.ai_base_url, DEFAULT_AI_BASE);
        assert_eq!(cfg.consulta.ai_model, DEFAULT_AI_MODEL);
        assert_eq!(cfg.consulta.history_limit, HISTORY_DEFAULT);

        cfg.consulta.online_enabled = true;
        cfg.consulta.target_lang = Some("es".into());
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(!json.contains("OPENCODE_API_KEY"), "{json}");
        assert!(!json.contains("api_key"), "{json}");
        assert!(!json.contains("ai_key"), "{json}");
        let back: Config = serde_json::from_str(&json).unwrap();
        assert!(back.consulta.online_enabled);
        assert_eq!(back.consulta.target_lang.as_deref(), Some("es"));
    }

    #[test]
    fn config_vieja_sin_consulta_usa_defaults() {
        let dir = temp_dir("consulta-defaults");
        let path = dir.join("config.json");
        std::fs::write(&path, r#"{"settings":{"font_size":21.0}}"#).unwrap();
        let cfg = Config::load_from(&path);
        assert_eq!(cfg.settings.font_size, 21.0);
        assert!(!cfg.consulta.online_enabled);
        assert_eq!(cfg.consulta.ai_model, DEFAULT_AI_MODEL);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn proveedor_invalido_cae_en_opencode() {
        let mut c = ConsultaSettings {
            ai_provider: "no-existe".into(),
            ai_base_url: String::new(),
            ai_model: String::new(),
            ..ConsultaSettings::default()
        };
        c.clamp();
        assert_eq!(c.ai_provider, "opencode");
        assert_eq!(c.ai_base_url, DEFAULT_AI_BASE);
        assert_eq!(c.ai_model, DEFAULT_AI_MODEL);
    }

    #[test]
    fn consulta_se_clampa() {
        let mut c = ConsultaSettings {
            ai_timeout_secs: 999,
            ai_max_tokens: 1,
            history_limit: 99_999,
            ai_base_url: " ".into(),
            ai_model: String::new(),
            translation_provider: String::new(),
            translation_model: String::new(),
            ..ConsultaSettings::default()
        };
        c.clamp();
        assert_eq!(c.ai_timeout_secs, 120);
        assert_eq!(c.ai_max_tokens, 64);
        assert_eq!(c.history_limit, 2000);
        assert_eq!(c.ai_base_url, DEFAULT_AI_BASE);
        assert_eq!(c.ai_model, DEFAULT_AI_MODEL);
        assert_eq!(c.translation_provider, "mymemory");
    }

    #[test]
    fn traduccion_local_completa_el_modelo() {
        let mut c = ConsultaSettings {
            translation_provider: "ollama".into(),
            translation_model: String::new(),
            ..ConsultaSettings::default()
        };
        c.clamp();
        assert_eq!(c.translation_provider, "ollama");
        assert_eq!(c.translation_model, "llama3.2");
    }
}
