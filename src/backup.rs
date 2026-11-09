//! Respaldo local de los datos de usuario en un ZIP.
//!
//! Incluye `config.json`, `library.json`, `annotations.json` y
//! `consultas.json`. No incluye los EPUB, las portadas cacheadas ni las claves
//! (que viven en el llavero). La importación valida el manifiesto y cada JSON,
//! y guarda un respaldo del estado actual antes de reemplazar nada.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::annotations::Annotations;
use crate::config::Config;
use crate::consulta::ConsultaHistory;
use crate::library::Library;
use crate::store;
use crate::update::sha256_hex;

pub const FORMAT_VERSION: u32 = 1;
pub const MANIFEST: &str = "candil-backup.json";
pub const DATA_FILES: &[&str] = &[
    "config.json",
    "library.json",
    "annotations.json",
    "consultas.json",
];
const MAX_FILE: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: u32,
    pub app_version: String,
    pub created: i64,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct ExportResult {
    pub path: PathBuf,
    pub files: usize,
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub backup: Option<PathBuf>,
    pub files: usize,
}

pub fn default_name() -> String {
    "candil-datos.zip".to_string()
}

/// Empaqueta los datos existentes en `target`.
pub fn export(data_dir: &Path, target: &Path) -> Result<ExportResult, String> {
    if DATA_FILES.iter().any(|name| target.ends_with(name)) {
        return Err("Elegí un archivo distinto de los datos de Candil.".into());
    }
    let mut files = Vec::new();
    let mut contents = Vec::new();
    for name in DATA_FILES {
        let path = data_dir.join(name);
        if !path.is_file() {
            continue;
        }
        let bytes = fs::read(&path).map_err(|e| format!("No pude leer {name}: {e}"))?;
        files.push(FileEntry {
            name: (*name).to_string(),
            size: bytes.len() as u64,
            sha256: sha256_hex(&bytes),
        });
        contents.push((*name, bytes));
    }
    if contents.is_empty() {
        return Err("No hay datos para exportar.".into());
    }
    let manifest = Manifest {
        format: FORMAT_VERSION,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        created: now(),
        files,
    };
    let out =
        File::create(target).map_err(|e| format!("No pude crear {}: {e}", target.display()))?;
    let mut writer = ZipWriter::new(out);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let manifest_bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|e| format!("No pude serializar: {e}"))?;
    writer
        .start_file(MANIFEST, options)
        .map_err(|e| format!("No pude escribir el manifiesto: {e}"))?;
    writer
        .write_all(&manifest_bytes)
        .map_err(|e| format!("No pude escribir el manifiesto: {e}"))?;
    for (name, bytes) in &contents {
        writer
            .start_file(*name, options)
            .map_err(|e| format!("No pude escribir {name}: {e}"))?;
        writer
            .write_all(bytes)
            .map_err(|e| format!("No pude escribir {name}: {e}"))?;
    }
    writer
        .finish()
        .map_err(|e| format!("No pude cerrar el respaldo: {e}"))?;
    Ok(ExportResult {
        path: target.to_path_buf(),
        files: manifest.files.len(),
    })
}

/// Valida el ZIP y reemplaza los datos. Antes crea un respaldo del estado
/// actual en `data_dir/backups/pre-import-<nanos>.zip`.
pub fn import(archive_path: &Path, data_dir: &Path) -> Result<ImportResult, String> {
    let file = File::open(archive_path)
        .map_err(|e| format!("No pude abrir {}: {e}", archive_path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("No es un respaldo válido de Candil: {e}"))?;
    let manifest_bytes = read_entry(&mut archive, MANIFEST)?;
    let manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).map_err(|e| format!("Manifiesto inválido: {e}"))?;
    if manifest.format == 0 || manifest.format > FORMAT_VERSION {
        return Err(format!(
            "El respaldo usa un formato no soportado ({}).",
            manifest.format
        ));
    }
    let mut imported = Vec::new();
    for entry in &manifest.files {
        if !DATA_FILES.contains(&entry.name.as_str()) {
            return Err(format!(
                "Archivo no permitido en el respaldo: {}",
                entry.name
            ));
        }
        let bytes = read_entry(&mut archive, &entry.name)?;
        if bytes.len() as u64 != entry.size {
            return Err(format!("El tamaño de {} no coincide.", entry.name));
        }
        if !entry.sha256.eq_ignore_ascii_case(&sha256_hex(&bytes)) {
            return Err(format!("El checksum de {} no coincide.", entry.name));
        }
        validate_json(&entry.name, &bytes)?;
        imported.push((entry.name.clone(), bytes));
    }
    if imported.is_empty() {
        return Err("El respaldo no contiene datos.".into());
    }

    let has_existing = DATA_FILES.iter().any(|name| data_dir.join(name).is_file());
    let backup = if has_existing {
        let dir = data_dir.join("backups");
        fs::create_dir_all(&dir).map_err(|e| format!("No pude crear {}: {e}", dir.display()))?;
        let path = dir.join(format!("pre-import-{}.zip", now_nanos()));
        export(data_dir, &path)?;
        Some(path)
    } else {
        None
    };

    fs::create_dir_all(data_dir)
        .map_err(|e| format!("No pude crear {}: {e}", data_dir.display()))?;
    for (name, bytes) in &imported {
        store::write_atomic(&data_dir.join(name), bytes)
            .map_err(|e| format!("No pude restaurar {name}: {e}"))?;
    }
    Ok(ImportResult {
        backup,
        files: imported.len(),
    })
}

fn read_entry(archive: &mut ZipArchive<File>, name: &str) -> Result<Vec<u8>, String> {
    let entry = archive
        .by_name(name)
        .map_err(|_| format!("Falta {name} en el respaldo."))?;
    if entry.size() > MAX_FILE {
        return Err(format!("{name} es demasiado grande para un respaldo."));
    }
    let mut bytes = Vec::new();
    entry
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("No pude leer {name}: {e}"))?;
    if bytes.len() as u64 > MAX_FILE {
        return Err(format!("{name} es demasiado grande para un respaldo."));
    }
    Ok(bytes)
}

fn validate_json(name: &str, bytes: &[u8]) -> Result<(), String> {
    let result = match name {
        "config.json" => serde_json::from_slice::<Config>(bytes).map(|_| ()),
        "library.json" => serde_json::from_slice::<Library>(bytes).map(|_| ()),
        "annotations.json" => serde_json::from_slice::<Annotations>(bytes).map(|_| ()),
        "consultas.json" => serde_json::from_slice::<ConsultaHistory>(bytes).map(|_| ()),
        _ => return Err(format!("Archivo no permitido: {name}")),
    };
    result.map_err(|e| format!("{name} inválido: {e}"))
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let stamp = now_nanos();
        let dir = std::env::temp_dir().join(format!(
            "candil-backup-{tag}-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_data(dir: &Path) {
        fs::write(
            dir.join("config.json"),
            serde_json::to_vec(&Config::default()).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("library.json"),
            serde_json::to_vec(&Library::default()).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("annotations.json"),
            serde_json::to_vec(&Annotations::default()).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("consultas.json"),
            serde_json::to_vec(&ConsultaHistory::default()).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn exporta_e_importa_los_cuatro_archivos() {
        let data = temp_dir("export");
        let restore = temp_dir("restore");
        write_data(&data);
        let archive = data.parent().unwrap().join(format!(
            "candil-test-{}-{}.zip",
            std::process::id(),
            now_nanos()
        ));
        let exported = export(&data, &archive).expect("exportar");
        assert_eq!(exported.files, 4);
        assert!(archive.is_file());

        let imported = import(&archive, &restore).expect("importar");
        assert_eq!(imported.files, 4);
        assert!(imported.backup.is_none());
        for name in DATA_FILES {
            assert!(restore.join(name).is_file(), "falta {name}");
            assert_eq!(
                fs::read(data.join(name)).unwrap(),
                fs::read(restore.join(name)).unwrap(),
                "{name}"
            );
        }
        let _ = fs::remove_dir_all(data);
        let _ = fs::remove_dir_all(restore);
        let _ = fs::remove_file(archive);
    }

    #[test]
    fn importar_guarda_un_backup_previo_si_habia_datos() {
        let data = temp_dir("previous");
        let restore = temp_dir("previous-restore");
        write_data(&data);
        write_data(&restore);
        let archive = data.parent().unwrap().join(format!(
            "candil-test-{}-{}.zip",
            std::process::id(),
            now_nanos()
        ));
        export(&data, &archive).expect("exportar");
        let imported = import(&archive, &restore).expect("importar");
        let backup = imported.backup.expect("backup previo");
        assert!(backup.is_file());
        assert!(backup
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("pre-import-"));
        let _ = fs::remove_dir_all(data);
        let _ = fs::remove_dir_all(restore);
        let _ = fs::remove_file(archive);
        let _ = fs::remove_dir_all(backup.parent().unwrap());
    }

    #[test]
    fn rechaza_checksum_incorrecto_sin_tocar_el_destino() {
        let data = temp_dir("checksum");
        write_data(&data);
        let bytes = fs::read(data.join("config.json")).unwrap();
        let manifest = Manifest {
            format: FORMAT_VERSION,
            app_version: "test".into(),
            created: now(),
            files: vec![FileEntry {
                name: "config.json".into(),
                size: bytes.len() as u64,
                sha256: "0".repeat(64),
            }],
        };
        let archive = data.parent().unwrap().join(format!(
            "candil-bad-{}-{}.zip",
            std::process::id(),
            now_nanos()
        ));
        let out = File::create(&archive).unwrap();
        let mut writer = ZipWriter::new(out);
        let options = SimpleFileOptions::default();
        writer.start_file(MANIFEST, options).unwrap();
        writer
            .write_all(&serde_json::to_vec(&manifest).unwrap())
            .unwrap();
        writer.start_file("config.json", options).unwrap();
        writer.write_all(&bytes).unwrap();
        writer.finish().unwrap();

        let restore = temp_dir("checksum-restore");
        let marker = restore.join("library.json");
        fs::write(&marker, b"no tocar").unwrap();
        let error = import(&archive, &restore).expect_err("debe fallar");
        assert!(error.contains("checksum"), "{error}");
        assert_eq!(fs::read(&marker).unwrap(), b"no tocar");
        let _ = fs::remove_dir_all(data);
        let _ = fs::remove_dir_all(restore);
        let _ = fs::remove_file(archive);
    }

    #[test]
    fn rechaza_archivos_fuera_de_la_lista() {
        let data = temp_dir("evil");
        write_data(&data);
        let bytes = fs::read(data.join("config.json")).unwrap();
        let manifest = Manifest {
            format: FORMAT_VERSION,
            app_version: "test".into(),
            created: now(),
            files: vec![FileEntry {
                name: "../config.json".into(),
                size: bytes.len() as u64,
                sha256: sha256_hex(&bytes),
            }],
        };
        let archive = data.parent().unwrap().join(format!(
            "candil-evil-{}-{}.zip",
            std::process::id(),
            now_nanos()
        ));
        let out = File::create(&archive).unwrap();
        let mut writer = ZipWriter::new(out);
        let options = SimpleFileOptions::default();
        writer.start_file(MANIFEST, options).unwrap();
        writer
            .write_all(&serde_json::to_vec(&manifest).unwrap())
            .unwrap();
        writer.start_file("../config.json", options).unwrap();
        writer.write_all(&bytes).unwrap();
        writer.finish().unwrap();

        let restore = temp_dir("evil-restore");
        let error = import(&archive, &restore).expect_err("debe fallar");
        assert!(error.contains("no permitido"), "{error}");
        let _ = fs::remove_dir_all(data);
        let _ = fs::remove_dir_all(restore);
        let _ = fs::remove_file(archive);
    }
}
