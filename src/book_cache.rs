//! Caché local y descartable de EPUB en montajes remotos.
//!
//! Leer un EPUB desde rclone/FUSE en cada apertura o búsqueda es caro. Esta
//! caché copia el archivo a `~/.cache/candil/books` la primera vez y reutiliza
//! la copia mientras tamaño y mtime coincidan. La ruta original sigue siendo la
//! identidad del libro: progreso, anotaciones y biblioteca no cambian.
//!
//! No entra en el respaldo de datos y se puede borrar sin perder nada.

use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::store;

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_LIMIT_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct CacheIndex {
    schema_version: u32,
    entries: Vec<CacheEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct CacheEntry {
    path: String,
    size: u64,
    mtime: i64,
    file: String,
    bytes: u64,
    last_used: i64,
}

pub struct BookCache {
    dir: PathBuf,
    limit: u64,
    index: Mutex<CacheIndex>,
}

impl Default for BookCache {
    fn default() -> Self {
        Self::new(DEFAULT_LIMIT_BYTES)
    }
}

impl BookCache {
    pub fn new(limit: u64) -> Self {
        let dir = dirs::cache_dir()
            .unwrap_or_else(|| store::data_dir().join("cache"))
            .join("candil")
            .join("books");
        Self::with_dir(dir, limit)
    }

    pub fn with_dir(dir: PathBuf, limit: u64) -> Self {
        let _ = fs::create_dir_all(&dir);
        let index = load_index(&dir);
        let cache = Self {
            dir,
            limit,
            index: Mutex::new(index),
        };
        let _ = cache.save_index();
        cache
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn total_bytes(&self) -> u64 {
        self.index
            .lock()
            .map(|index| index.entries.iter().map(|entry| entry.bytes).sum())
            .unwrap_or(0)
    }

    pub fn entry_count(&self) -> usize {
        self.index
            .lock()
            .map(|index| index.entries.len())
            .unwrap_or(0)
    }

    /// `true` si la ruta vive en un montaje remoto y por lo tanto conviene
    /// copiarla a la caché local.
    pub fn is_remote(&self, path: &Path) -> bool {
        is_remote_path(path) && !path.starts_with(&self.dir)
    }

    pub fn clear(&self) -> Result<u64, String> {
        let mut index = self
            .index
            .lock()
            .map_err(|_| "No pude bloquear la caché.".to_string())?;
        let freed = index.entries.iter().map(|entry| entry.bytes).sum();
        index.entries.clear();
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(path);
                }
            }
        }
        let _ = store::save_json_atomic(&self.dir.join("index.json"), &*index);
        Ok(freed)
    }

    /// Devuelve la copia local si el libro vive en un montaje remoto y la caché
    /// es válida. Si el libro es local, o la copia falla, devuelve `None` para
    /// que el llamador abra el original.
    pub fn ensure(&self, path: &Path) -> Option<PathBuf> {
        if !is_remote_path(path) || path.starts_with(&self.dir) {
            return None;
        }
        let (size, mtime) = stamp(path)?;
        self.ensure_stamped(path, size, mtime).ok().flatten()
    }

    #[cfg(test)]
    fn ensure_for_test(&self, path: &Path) -> Option<PathBuf> {
        let (size, mtime) = stamp(path)?;
        self.ensure_stamped(path, size, mtime).ok().flatten()
    }

    fn ensure_stamped(
        &self,
        path: &Path,
        size: u64,
        mtime: i64,
    ) -> Result<Option<PathBuf>, String> {
        let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let key = canonical.to_string_lossy().to_string();
        {
            let mut index = self
                .index
                .lock()
                .map_err(|_| "No pude bloquear la caché.".to_string())?;
            if let Some(entry) = index
                .entries
                .iter_mut()
                .find(|entry| entry.path == key && entry.size == size && entry.mtime == mtime)
            {
                let target = self.dir.join(&entry.file);
                if target.is_file() {
                    entry.last_used = now();
                    let _ = store::save_json_atomic(&self.dir.join("index.json"), &*index);
                    return Ok(Some(target));
                }
            }
            index.entries.retain(|entry| {
                !(entry.path == key && (entry.size != size || entry.mtime != mtime))
            });
        }

        if size > self.limit {
            return Ok(None);
        }
        let file_name = format!("{}-{}-{}.epub", short_hash(&key), size, mtime.max(0));
        let target = self.dir.join(&file_name);
        if !target.is_file() {
            let temp = self
                .dir
                .join(format!(".{file_name}.{}.tmp", std::process::id()));
            let copy = || -> Result<(), String> {
                let mut source =
                    File::open(&canonical).map_err(|e| format!("No pude abrir el EPUB: {e}"))?;
                let mut output =
                    File::create(&temp).map_err(|e| format!("No pude crear la caché: {e}"))?;
                std::io::copy(&mut source, &mut output)
                    .map_err(|e| format!("No pude copiar a la caché: {e}"))?;
                output
                    .sync_all()
                    .map_err(|e| format!("No pude sincronizar la caché: {e}"))?;
                fs::rename(&temp, &target).map_err(|e| format!("No pude guardar la caché: {e}"))
            };
            if let Err(error) = copy() {
                let _ = fs::remove_file(&temp);
                return Err(error);
            }
        }

        let bytes = fs::metadata(&target).map(|meta| meta.len()).unwrap_or(size);
        let mut index = self
            .index
            .lock()
            .map_err(|_| "No pude bloquear la caché.".to_string())?;
        let old_file = index
            .entries
            .iter()
            .find(|entry| entry.path == key)
            .map(|entry| entry.file.clone());
        index.entries.retain(|entry| entry.path != key);
        if let Some(old_file) = old_file {
            if old_file != file_name {
                let _ = fs::remove_file(self.dir.join(old_file));
            }
        }
        index.entries.push(CacheEntry {
            path: key,
            size,
            mtime,
            file: file_name.clone(),
            bytes,
            last_used: now(),
        });
        while index.entries.iter().map(|entry| entry.bytes).sum::<u64>() > self.limit {
            let Some((position, _)) = index
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.file != file_name)
                .min_by_key(|(_, entry)| entry.last_used)
            else {
                break;
            };
            let removed = index.entries.remove(position);
            let _ = fs::remove_file(self.dir.join(removed.file));
        }
        let current_kept = index.entries.iter().any(|entry| entry.file == file_name);
        let _ = store::save_json_atomic(&self.dir.join("index.json"), &*index);
        if current_kept && target.is_file() {
            Ok(Some(target))
        } else {
            Ok(None)
        }
    }

    fn save_index(&self) -> Result<(), String> {
        let index = self
            .index
            .lock()
            .map_err(|_| "No pude bloquear la caché.".to_string())?;
        store::save_json_atomic(&self.dir.join("index.json"), &*index)
    }
}

fn load_index(dir: &Path) -> CacheIndex {
    let path = dir.join("index.json");
    let mut index: CacheIndex = fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    index.schema_version = SCHEMA_VERSION;
    index
        .entries
        .retain(|entry| dir.join(&entry.file).is_file());
    let known: HashSet<&str> = index
        .entries
        .iter()
        .map(|entry| entry.file.as_str())
        .collect();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "index.json" || name.ends_with(".tmp") {
                let _ = fs::remove_file(path);
                continue;
            }
            if path.is_file() && !known.contains(name.as_str()) {
                let _ = fs::remove_file(path);
            }
        }
    }
    index
}

fn stamp(path: &Path) -> Option<(u64, i64)> {
    let metadata = fs::metadata(path).ok()?;
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    Some((metadata.len(), mtime))
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn short_hash(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    let mut out = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(target_os = "linux")]
fn is_remote_path(path: &Path) -> bool {
    use std::sync::OnceLock;
    static REMOTES: OnceLock<Vec<(PathBuf, String)>> = OnceLock::new();
    let remotes = REMOTES.get_or_init(parse_mounts);
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    remotes
        .iter()
        .filter(|(mount, _)| path.starts_with(mount))
        .max_by_key(|(mount, _)| mount.components().count())
        .is_some_and(|(_, fstype)| is_remote_fs(fstype))
}

#[cfg(target_os = "linux")]
fn parse_mounts() -> Vec<(PathBuf, String)> {
    let Ok(text) = fs::read_to_string("/proc/mounts") else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let _device = fields.next()?;
            let mount = unescape_mount(fields.next()?);
            let fstype = fields.next()?.to_string();
            Some((PathBuf::from(mount), fstype))
        })
        .collect()
}

#[cfg(target_os = "linux")]
fn unescape_mount(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

#[cfg(target_os = "linux")]
fn is_remote_fs(fstype: &str) -> bool {
    let fstype = fstype.to_ascii_lowercase();
    if fstype.starts_with("fuse") {
        return true;
    }
    matches!(
        fstype.as_str(),
        "nfs"
            | "nfs4"
            | "cifs"
            | "smb3"
            | "sshfs"
            | "davfs"
            | "9p"
            | "glusterfs"
            | "ceph"
            | "afs"
            | "webdav"
    )
}

#[cfg(target_os = "macos")]
fn is_remote_path(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.starts_with("/Volumes/") || text.contains("/Library/CloudStorage/")
}

#[cfg(target_os = "windows")]
fn is_remote_path(path: &Path) -> bool {
    path.to_string_lossy().starts_with("\\\\")
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn is_remote_path(_path: &Path) -> bool {
    false
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
            std::env::temp_dir().join(format!("candil-cache-{tag}-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copia_y_reutiliza_si_el_archivo_no_cambio() {
        let root = temp_dir("hit");
        let source = root.join("libro.epub");
        fs::write(&source, b"contenido").unwrap();
        let cache = BookCache::with_dir(root.join("cache"), 1024 * 1024);
        let first = cache.ensure_for_test(&source).expect("cachea");
        assert_eq!(fs::read(&first).unwrap(), b"contenido");
        let second = cache.ensure_for_test(&source).expect("reutiliza");
        assert_eq!(first, second);
        assert_eq!(cache.total_bytes(), 9);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalida_al_cambiar_el_archivo() {
        let root = temp_dir("stale");
        let source = root.join("libro.epub");
        fs::write(&source, b"uno").unwrap();
        let cache = BookCache::with_dir(root.join("cache"), 1024 * 1024);
        let first = cache.ensure_for_test(&source).unwrap();
        fs::write(&source, b"contenido distinto").unwrap();
        let second = cache.ensure_for_test(&source).unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(&second).unwrap(), b"contenido distinto");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn respeta_el_limite_y_borra_lo_mas_viejo() {
        let root = temp_dir("limit");
        let cache = BookCache::with_dir(root.join("cache"), 10);
        let a = root.join("a.epub");
        let b = root.join("b.epub");
        fs::write(&a, b"123456").unwrap();
        fs::write(&b, b"abcdef").unwrap();
        let first = cache.ensure_for_test(&a).unwrap();
        let second = cache.ensure_for_test(&b).unwrap();
        assert!(!first.exists(), "el más viejo debe salir");
        assert!(second.exists(), "el último debe quedar");
        assert!(cache.total_bytes() <= 10);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn clear_borra_los_archivos() {
        let root = temp_dir("clear");
        let source = root.join("libro.epub");
        fs::write(&source, b"contenido").unwrap();
        let cache = BookCache::with_dir(root.join("cache"), 1024 * 1024);
        let cached = cache.ensure_for_test(&source).unwrap();
        assert!(cached.exists());
        cache.clear().unwrap();
        assert!(!cached.exists());
        assert_eq!(cache.total_bytes(), 0);
        let _ = fs::remove_dir_all(root);
    }
}
