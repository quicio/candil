//! Persistencia JSON compartida: escritura atómica y arranque tolerante.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Directorio de datos de Candil (`.../candil`), donde viven `config.json`,
/// `library.json`, `annotations.json`, `consultas.json` y `covers/`.
pub fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        resolve_data_dir(&base)
    })
    .clone()
}

/// Migra una instalación previa llamada `lector` a `candil` sin perder datos.
/// Si el rename falla (otro disco, permisos), seguimos usando `lector` para no
/// arrancar con una biblioteca vacía.
fn resolve_data_dir(base: &Path) -> PathBuf {
    let new = base.join("candil");
    let old = base.join("lector");
    if !new.exists() && old.is_dir() {
        if std::fs::rename(&old, &new).is_ok() {
            return new;
        }
        eprintln!(
            "no pude migrar {} a {}; sigo usando el directorio viejo",
            old.display(),
            new.display()
        );
        return old;
    }
    new
}

/// Escribe `<archivo>.tmp`, sincroniza y reemplaza: un corte no deja el JSON a
/// medias. El temporal queda en el mismo directorio para que `rename` sea atómico.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("no pude crear {}: {e}", parent.display()))?;
    }
    let tmp = path.with_extension("json.tmp");
    let mut file =
        std::fs::File::create(&tmp).map_err(|e| format!("no pude crear {}: {e}", tmp.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("no pude escribir {}: {e}", tmp.display()))?;
    file.sync_all()
        .map_err(|e| format!("no pude sincronizar {}: {e}", tmp.display()))?;
    drop(file);
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("no pude reemplazar {}: {e}", path.display())
    })
}

pub fn save_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let json = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("no pude serializar {}: {e}", path.display()))?;
    write_atomic(path, &json)
}

/// Carga un JSON. Si está corrupto, lo renombra a `<archivo>.corrupt-<epoch>`,
/// devuelve el default y un mensaje para mostrar en la UI.
pub fn load_json_with_report<T: DeserializeOwned + Default>(
    path: &Path,
    label: &str,
) -> (T, Option<String>) {
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) => {
            let report = path.exists().then(|| format!("no pude leer {label}: {e}"));
            return (T::default(), report);
        }
    };
    match serde_json::from_str::<T>(&raw) {
        Ok(value) => (value, None),
        Err(e) => {
            let secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let backup = path.with_extension(format!("json.corrupt-{secs}"));
            let report = match std::fs::rename(path, &backup) {
                Ok(()) => {
                    let name = backup
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "archivo.corrupt".into());
                    eprintln!("{label} ilegible: {e}");
                    format!("{label} ilegible; guardé una copia como {name}")
                }
                Err(_) => {
                    eprintln!("{label} ilegible: {e}");
                    format!("{label} ilegible; no pude guardar una copia")
                }
            };
            (T::default(), Some(report))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(tag: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("candil-store-{tag}-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn migra_lector_a_candil_sin_perder_datos() {
        let base = temp("migrate");
        let old = base.join("lector");
        std::fs::create_dir_all(old.join("covers")).unwrap();
        std::fs::write(old.join("config.json"), "{}").unwrap();
        std::fs::write(old.join("covers").join("a.png"), b"png").unwrap();

        let resolved = resolve_data_dir(&base);
        assert_eq!(resolved, base.join("candil"));
        assert!(!old.exists());
        assert!(base.join("candil/config.json").is_file());
        assert!(base.join("candil/covers/a.png").is_file());
        let _ = std::fs::remove_dir_all(base);
    }

    #[test]
    fn si_ya_existe_candil_no_pisa_nada() {
        let base = temp("both");
        std::fs::create_dir_all(base.join("lector")).unwrap();
        std::fs::create_dir_all(base.join("candil")).unwrap();
        std::fs::write(base.join("candil/config.json"), "new").unwrap();
        std::fs::write(base.join("lector/config.json"), "old").unwrap();

        let resolved = resolve_data_dir(&base);
        assert_eq!(resolved, base.join("candil"));
        assert_eq!(
            std::fs::read_to_string(base.join("candil/config.json")).unwrap(),
            "new"
        );
        assert!(base.join("lector").is_dir());
        let _ = std::fs::remove_dir_all(base);
    }
}
