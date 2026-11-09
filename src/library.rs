//! Biblioteca: índice persistente de carpetas y libros, sin copiar archivos.
//!
//! El disco manda: `library.json` guarda rutas canónicas y metadatos; las
//! portadas se cachean en `covers/` y el escaneo es incremental por tamaño y
//! `mtime`. Ver ADR-0007 y ADR-0008.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::{Config, Progress, Settings};
use crate::covers;
use crate::epub::Book;
use crate::store;

pub const SCHEMA_VERSION: u32 = 1;

/// Versión de extracción de metadatos. Un libro con una versión anterior se
/// vuelve a leer en el próximo escaneo aunque el archivo no haya cambiado.
pub const META_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub(crate) fn now() -> i64 {
    now_secs()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BookEntry {
    /// Ruta absoluta canónica; también es la clave de identidad.
    pub path: String,
    pub title: String,
    pub author: String,
    pub lang: String,
    /// Serie/colección declarada en el OPF, si existe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    /// Posición dentro de la serie.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_index: Option<f32>,
    /// Versión de metadatos con la que se indexó la entrada.
    #[serde(default)]
    pub meta_version: u32,
    /// Ruta relativa al directorio de datos: `covers/<hash>.png`.
    pub cover: Option<String>,
    pub size: u64,
    /// Segundos desde UNIX_EPOCH.
    pub mtime: i64,
    pub chapters: usize,
    pub progress: Progress,
    /// Overrides de tipografía/tema solo para este libro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<Settings>,
    pub added: i64,
    pub last_opened: Option<i64>,
    pub missing: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Library {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub folders: Vec<String>,
    pub books: Vec<BookEntry>,
    #[serde(skip)]
    pub data_dir: PathBuf,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            folders: Vec::new(),
            books: Vec::new(),
            data_dir: store::data_dir(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScanStats {
    pub found: usize,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub failed: usize,
}

impl ScanStats {
    pub fn add(&mut self, other: ScanStats) {
        self.found += other.found;
        self.added += other.added;
        self.updated += other.updated;
        self.unchanged += other.unchanged;
        self.failed += other.failed;
    }
}

impl BookEntry {
    /// Metadatos + `size`/`mtime`; sin extraer portada.
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        Self::from_path_with_cover(path, None)
    }

    /// Igual que `from_path`, pero genera la miniatura en `covers_dir` si hay
    /// portada. `covers_dir` se inyecta para que los tests no escriban en la
    /// configuración real.
    pub fn from_path_with_cover(path: impl AsRef<Path>, covers_dir: Option<&Path>) -> Self {
        let raw = path.as_ref();
        let canonical = std::fs::canonicalize(raw).unwrap_or_else(|_| raw.to_path_buf());
        let (size, mtime) = file_stamp(&canonical);
        let mut entry = Self {
            path: canonical.to_string_lossy().to_string(),
            title: file_stem(&canonical),
            size,
            mtime,
            added: now_secs(),
            meta_version: META_VERSION,
            ..Self::default()
        };

        match Book::open(&canonical) {
            Ok(mut book) => {
                if !book.title.trim().is_empty() {
                    entry.title = book.title.clone();
                }
                entry.author = book.author.clone();
                entry.lang = book.lang.clone();
                entry.series = book.series.clone();
                entry.series_index = book.series_index;
                entry.chapters = book.chapter_count();
                if let Some(dir) = covers_dir {
                    if let Some(bytes) = book.cover_bytes() {
                        if let Some(png) = covers::thumbnail(&bytes, covers::COVER_HEIGHT) {
                            let target = covers::cover_path_for(dir, &canonical);
                            if covers::write_cover(&target, &png).is_ok() {
                                if let Some(name) = target.file_name() {
                                    entry.cover =
                                        Some(format!("covers/{}", name.to_string_lossy()));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                entry.error = Some(e);
            }
        }
        entry
    }

    pub fn effective_settings(&self, global: Settings) -> Settings {
        let mut settings = self.settings.unwrap_or(global);
        settings.clamp();
        settings
    }
}

impl Library {
    pub fn file() -> PathBuf {
        store::data_dir().join("library.json")
    }

    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            ..Self::default()
        }
    }

    pub fn load() -> Self {
        Self::load_with_report().0
    }

    pub fn load_with_report() -> (Self, Option<String>) {
        Self::load_with_report_from(&Self::file())
    }

    pub fn load_from(path: &Path) -> Self {
        Self::load_with_report_from(path).0
    }

    pub fn load_with_report_from(path: &Path) -> (Self, Option<String>) {
        let (mut lib, report) = store::load_json_with_report::<Self>(path, "biblioteca");
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            lib.data_dir = parent.to_path_buf();
        }
        if lib.schema_version == 0 {
            lib.schema_version = SCHEMA_VERSION;
        }
        (lib, report)
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::file())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        store::save_json_atomic(path, self)
    }

    pub fn upsert(&mut self, entry: BookEntry) {
        if let Some(index) = self.books.iter().position(|b| b.path == entry.path) {
            self.books[index] = entry;
        } else {
            self.books.push(entry);
        }
    }

    /// Reasocia una entrada con otro archivo conservando progreso, ajustes y
    /// fechas. Devuelve la ruta canónica nueva. No toca anotaciones ni
    /// `config.recent`: el llamador decide si migra esas referencias.
    pub fn relink(&mut self, index: usize, path: &Path) -> Result<String, String> {
        let Some(old) = self.books.get(index).cloned() else {
            return Err("La entrada ya no existe.".into());
        };
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let key = canonical.to_string_lossy().to_string();
        if self
            .books
            .iter()
            .enumerate()
            .any(|(i, b)| i != index && b.path == key)
        {
            return Err("Ese EPUB ya está en la biblioteca.".into());
        }
        let mut entry =
            BookEntry::from_path_with_cover(&canonical, Some(&self.data_dir.join("covers")));
        if let Some(error) = entry.error.clone() {
            return Err(format!("No pude abrir el EPUB: {error}"));
        }
        entry.progress = old.progress;
        entry.settings = old.settings;
        entry.added = old.added;
        entry.last_opened = old.last_opened;
        let new_path = entry.path.clone();
        self.books[index] = entry;
        Ok(new_path)
    }

    /// Migra `config.progress` a las entradas de la biblioteca y vacía las
    /// claves migradas. No pisa un progreso que la biblioteca ya tenga.
    pub fn migrate_progress_from(&mut self, cfg: &mut Config) -> usize {
        if cfg.progress.is_empty() || self.books.is_empty() {
            return 0;
        }
        let keys: Vec<String> = cfg.progress.keys().cloned().collect();
        let mut migrated = 0;
        for key in keys {
            let canonical = std::fs::canonicalize(&key)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| key.clone());
            let Some(index) = self
                .books
                .iter()
                .position(|b| b.path == canonical || b.path == key)
            else {
                continue;
            };
            let Some(progress) = cfg.progress.remove(&key) else {
                continue;
            };
            let entry = &mut self.books[index];
            if entry.progress.is_empty() {
                entry.progress = progress;
                migrated += 1;
            }
        }
        migrated
    }

    /// Deja de vigilar una carpeta. Las entradas de sus libros se conservan:
    /// quitar una carpeta no borra nada del disco ni de la biblioteca.
    pub fn unwatch_folder(&mut self, folder: &str) {
        self.folders.retain(|f| f != folder);
    }

    /// Escanea una carpeta recursivamente, sin seguir symlinks. La primera vez
    /// abre cada EPUB; las siguientes comparan `size` + `mtime`.
    pub fn scan_folder(&mut self, dir: &Path) -> ScanStats {
        let mut stats = ScanStats::default();
        if !dir.is_dir() {
            return stats;
        }

        let root = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        let root_key = root.to_string_lossy().to_string();
        if !self.folders.iter().any(|f| f == &root_key) {
            self.folders.push(root_key);
        }

        let covers_dir = self.data_dir.join("covers");
        let mut files = Vec::new();
        collect_epubs(&root, &mut files);
        files.sort();
        let mut seen: HashSet<String> = HashSet::new();

        for path in files {
            stats.found += 1;
            let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            let key = canonical.to_string_lossy().to_string();
            seen.insert(key.clone());
            let (size, mtime) = file_stamp(&canonical);

            if let Some(index) = self.books.iter().position(|b| b.path == key) {
                let (progress, added, last_opened, old_cover, old_settings, unchanged) = {
                    let old = &self.books[index];
                    let cover_ok = old
                        .cover
                        .as_ref()
                        .map(|cover| self.data_dir.join(cover).is_file())
                        .unwrap_or(true);
                    (
                        old.progress,
                        old.added,
                        old.last_opened,
                        old.cover.clone(),
                        old.settings,
                        old.size == size
                            && old.mtime == mtime
                            && old.meta_version >= META_VERSION
                            && old.chapters > 0
                            && old.error.is_none()
                            && !old.missing
                            && cover_ok,
                    )
                };
                if unchanged {
                    self.books[index].missing = false;
                    stats.unchanged += 1;
                    continue;
                }

                let mut entry =
                    BookEntry::from_path_with_cover(&canonical, Some(covers_dir.as_path()));
                entry.progress = progress;
                entry.added = added;
                entry.last_opened = last_opened;
                entry.settings = old_settings;
                if entry.cover.is_none() {
                    entry.cover = old_cover;
                }
                if entry.error.is_some() {
                    stats.failed += 1;
                }
                self.books[index] = entry;
                stats.updated += 1;
            } else {
                let entry = BookEntry::from_path_with_cover(&canonical, Some(covers_dir.as_path()));
                if entry.error.is_some() {
                    stats.failed += 1;
                }
                self.books.push(entry);
                stats.added += 1;
            }
        }

        for book in &mut self.books {
            if Path::new(&book.path).starts_with(&root) && !seen.contains(&book.path) {
                book.missing = true;
            }
        }
        stats
    }

    pub fn rescan(&mut self) -> ScanStats {
        let folders = self.folders.clone();
        let mut total = ScanStats::default();
        for folder in folders {
            total.add(self.scan_folder(Path::new(&folder)));
        }
        total
    }
}

fn file_stamp(path: &Path) -> (u64, i64) {
    let Ok(meta) = std::fs::metadata(path) else {
        return (0, 0);
    };
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    (size, mtime)
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Sin título".into())
}

fn is_epub(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("epub"))
        .unwrap_or(false)
}

fn collect_epubs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_epubs(&path, out);
        } else if meta.is_file() && is_epub(&path) {
            out.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("candil-lib-{tag}-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    #[test]
    fn metadatos_de_un_fixture() {
        let entry = BookEntry::from_path(fixtures().join("mini.epub"));
        assert_eq!(entry.title, "Libro de prueba");
        assert_eq!(entry.author, "Autora Ficticia");
        assert_eq!(entry.lang, "es");
        assert_eq!(entry.series.as_deref(), Some("Serie de prueba"));
        assert_eq!(entry.series_index, Some(2.0));
        assert_eq!(entry.meta_version, META_VERSION);
        assert!(entry.size > 0);
        assert!(entry.mtime > 0);
        assert_eq!(entry.chapters, 3);
        assert!(entry.error.is_none());
        assert!(Path::new(&entry.path).is_absolute());
        assert!(entry.cover.is_none(), "sin covers_dir no se extrae");
    }

    #[test]
    fn epub_corrupto_no_propaga_error() {
        let dir = temp_dir("corrupt-book");
        let path = dir.join("roto.epub");
        std::fs::write(&path, b"esto no es un zip").unwrap();

        let entry = BookEntry::from_path(&path);
        assert!(entry.error.is_some(), "{entry:?}");
        assert_eq!(entry.title, "roto");
        assert!(entry.size > 0);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn escaneo_conserva_settings_por_libro() {
        let data = temp_dir("scan-settings");
        let mut lib = Library::with_data_dir(data.clone());
        lib.scan_folder(&fixtures());
        let key = lib.books[0].path.clone();
        lib.books[0].settings = Some(Settings {
            font_size: 25.0,
            theme: 1,
            ..Settings::default()
        });

        // Camino sin cambios.
        lib.scan_folder(&fixtures());
        let entry = lib.books.iter().find(|b| b.path == key).unwrap();
        assert_eq!(entry.settings.map(|s| s.font_size), Some(25.0));

        // Camino de actualización: borrar la portada fuerza a reabrir el EPUB.
        if let Some(cover) = lib.books[0].cover.clone() {
            let _ = std::fs::remove_file(data.join(cover));
        }
        lib.scan_folder(&fixtures());
        let entry = lib.books.iter().find(|b| b.path == key).unwrap();
        assert_eq!(entry.settings.map(|s| s.theme), Some(1));

        let _ = std::fs::remove_dir_all(data);
    }

    #[test]
    fn escaneo_incremental_y_portadas() {
        let data = temp_dir("scan-data");
        let mut lib = Library::with_data_dir(data.clone());

        let first = lib.scan_folder(&fixtures());
        assert_eq!(
            first,
            ScanStats {
                found: 3,
                added: 3,
                updated: 0,
                unchanged: 0,
                failed: 0,
            }
        );
        assert_eq!(lib.books.len(), 3);
        assert!(
            lib.books.iter().all(|b| b.cover.is_some()),
            "{:?}",
            lib.books
        );
        assert_eq!(
            std::fs::read_dir(data.join("covers")).unwrap().count(),
            3,
            "una portada por libro"
        );

        let second = lib.scan_folder(&fixtures());
        assert_eq!(
            second,
            ScanStats {
                found: 3,
                added: 0,
                updated: 0,
                unchanged: 3,
                failed: 0,
            }
        );

        // Si se borra el caché, el libro se reabre y la portada se regenera.
        let cover_path = {
            let relative = lib
                .books
                .iter()
                .find_map(|b| b.cover.clone())
                .expect("algún libro tiene portada");
            data.join(relative)
        };
        std::fs::remove_file(&cover_path).unwrap();
        let third = lib.scan_folder(&fixtures());
        assert_eq!(third.found, 3);
        assert_eq!(third.updated, 1);
        assert_eq!(third.unchanged, 2);
        assert!(cover_path.is_file(), "la portada debe regenerarse");

        let path = data.join("library.json");
        lib.save_to(&path).expect("guardar biblioteca");
        let loaded = Library::load_from(&path);
        assert_eq!(loaded.books.len(), 3);
        assert_eq!(loaded.folders.len(), 1);
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);

        let _ = std::fs::remove_dir_all(data);
    }

    #[test]
    fn escaneo_reporta_libros_rotos() {
        let source = temp_dir("scan-broken-src");
        std::fs::copy(fixtures().join("mini.epub"), source.join("ok.epub")).unwrap();
        std::fs::write(source.join("roto.epub"), b"no es un epub").unwrap();

        let data = temp_dir("scan-broken-data");
        let mut lib = Library::with_data_dir(data.clone());
        let stats = lib.scan_folder(&source);
        assert_eq!(stats.found, 2);
        assert_eq!(stats.added, 2);
        assert_eq!(stats.failed, 1);
        assert!(lib.books.iter().any(|b| b.error.is_some()));

        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(data);
    }

    #[test]
    fn carpeta_inexistente_no_paniquea() {
        let mut lib = Library::default();
        let stats = lib.scan_folder(Path::new("/no/existe/lector-biblioteca"));
        assert_eq!(stats, ScanStats::default());
        assert!(lib.folders.is_empty());
    }

    #[test]
    fn marca_faltantes_al_reescanear() {
        let source = temp_dir("missing-src");
        std::fs::copy(fixtures().join("mini.epub"), source.join("a.epub")).unwrap();
        let data = temp_dir("missing-data");
        let mut lib = Library::with_data_dir(data.clone());

        let first = lib.scan_folder(&source);
        assert_eq!(first.added, 1);
        assert!(!lib.books[0].missing);

        std::fs::remove_file(source.join("a.epub")).unwrap();
        let second = lib.scan_folder(&source);
        assert_eq!(second.found, 0);
        assert!(lib.books[0].missing);

        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(data);
    }

    #[test]
    fn biblioteca_corrupta_hace_backup() {
        let data = temp_dir("corrupt-lib");
        let path = data.join("library.json");
        std::fs::write(&path, "{ esto no es json").unwrap();

        let (lib, report) = Library::load_with_report_from(&path);
        assert!(report.is_some());
        assert!(lib.books.is_empty());
        assert!(!path.exists());
        let backups: Vec<_> = std::fs::read_dir(&data)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".corrupt-"))
            .collect();
        assert_eq!(backups.len(), 1);

        let _ = std::fs::remove_dir_all(data);
    }

    #[test]
    fn json_viejo_sin_schema_version_carga() {
        let data = temp_dir("old-lib");
        let path = data.join("library.json");
        std::fs::write(&path, r#"{"folders":[],"books":[]}"#).unwrap();

        let lib = Library::load_from(&path);
        assert_eq!(lib.schema_version, SCHEMA_VERSION);

        let _ = std::fs::remove_dir_all(data);
    }

    #[test]
    fn relink_conserva_progreso_y_settings() {
        let data = temp_dir("relink-data");
        let mut lib = Library::with_data_dir(data.clone());
        let old = BookEntry::from_path(fixtures().join("mini.epub"));
        lib.books.push(old);
        lib.books[0].progress.chapter = 2;
        lib.books[0].settings = Some(Settings::default());
        let old_path = lib.books[0].path.clone();

        let new_path = lib
            .relink(0, &fixtures().join("edge.epub"))
            .expect("relink");
        assert_ne!(old_path, new_path);
        assert_eq!(lib.books[0].progress.chapter, 2);
        assert!(lib.books[0].settings.is_some());
        assert!(lib.books[0].error.is_none());

        let duplicate = BookEntry::from_path(fixtures().join("mini.epub"));
        lib.books.push(duplicate);
        assert!(lib.relink(1, &fixtures().join("edge.epub")).is_err());

        let _ = std::fs::remove_dir_all(data);
    }

    #[test]
    fn upsert_reemplaza_por_ruta() {
        let mut lib = Library::default();
        let mut entry = BookEntry {
            path: "/libro.epub".into(),
            title: "A".into(),
            ..BookEntry::default()
        };
        lib.upsert(entry.clone());
        entry.title = "B".into();
        lib.upsert(entry);

        assert_eq!(lib.books.len(), 1);
        assert_eq!(lib.books[0].title, "B");
    }

    #[test]
    fn migra_progreso_desde_config() {
        let mut lib = Library::default();
        lib.upsert(BookEntry {
            path: "/libro.epub".into(),
            ..BookEntry::default()
        });
        let mut cfg = Config::default();
        cfg.progress.insert(
            "/libro.epub".into(),
            Progress {
                chapter: 7,
                ratio: 0.5,
                anchor: Some(3),
                anchor_ratio: None,
                legacy_offset: None,
            },
        );

        assert_eq!(lib.migrate_progress_from(&mut cfg), 1);
        assert!(cfg.progress.is_empty());
        assert_eq!(lib.books[0].progress.chapter, 7);
        assert!((lib.books[0].progress.ratio - 0.5).abs() < f32::EPSILON);
        assert_eq!(lib.books[0].progress.anchor, Some(3));
    }

    #[test]
    fn migracion_no_pisa_progreso_y_conserva_huerfanos() {
        let mut lib = Library::default();
        lib.upsert(BookEntry {
            path: "/a.epub".into(),
            progress: Progress {
                chapter: 3,
                ratio: 0.2,
                anchor: Some(2),
                anchor_ratio: None,
                legacy_offset: None,
            },
            ..BookEntry::default()
        });
        let mut cfg = Config::default();
        cfg.progress.insert(
            "/a.epub".into(),
            Progress {
                chapter: 9,
                ratio: 0.9,
                anchor: None,
                anchor_ratio: None,
                legacy_offset: None,
            },
        );
        cfg.progress.insert(
            "/b.epub".into(),
            Progress {
                chapter: 1,
                ratio: 0.1,
                anchor: None,
                anchor_ratio: None,
                legacy_offset: None,
            },
        );

        assert_eq!(lib.migrate_progress_from(&mut cfg), 0);
        assert!(!cfg.progress.contains_key("/a.epub"));
        assert!(cfg.progress.contains_key("/b.epub"));
        assert_eq!(lib.books[0].progress.chapter, 3);
        assert!((lib.books[0].progress.ratio - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn dejar_de_vigilar_conserva_los_libros() {
        let mut lib = Library::default();
        lib.folders.push("/x".into());
        lib.upsert(BookEntry {
            path: "/x/a.epub".into(),
            ..BookEntry::default()
        });

        lib.unwatch_folder("/x");
        assert!(lib.folders.is_empty());
        assert_eq!(lib.books.len(), 1);
    }

    #[test]
    fn settings_por_libro_tienen_precedencia() {
        let global = Settings {
            font_size: 20.0,
            ..Settings::default()
        };
        let mut entry = BookEntry::default();
        assert_eq!(entry.effective_settings(global).font_size, 20.0);

        entry.settings = Some(Settings {
            font_size: 26.0,
            ..Settings::default()
        });
        assert_eq!(entry.effective_settings(global).font_size, 26.0);

        entry.settings = Some(Settings {
            font_size: 999.0,
            ..Settings::default()
        });
        assert_eq!(
            entry.effective_settings(global).font_size,
            crate::config::FONT_MAX
        );
    }
}
