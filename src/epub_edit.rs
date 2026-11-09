//! Edición acotada de un EPUB: metadatos Dublin Core y portada.
//!
//! Reescribe el ZIP completo para no perder entradas, valida el resultado con
//! [`Book::open`] y reemplaza el original solo después de guardar un backup.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::epub::{dir_of, read_entry, resolve, Book};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverExt {
    Jpg,
    Png,
}

impl CoverExt {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpg => "jpg",
            Self::Png => "png",
        }
    }

    pub fn media_type(self) -> &'static str {
        match self {
            Self::Jpg => "image/jpeg",
            Self::Png => "image/png",
        }
    }

    pub fn from_hint(media_type: &str, href: &str) -> Self {
        if media_type.eq_ignore_ascii_case("image/png")
            || href.to_ascii_lowercase().ends_with(".png")
        {
            Self::Png
        } else {
            Self::Jpg
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoverInfo {
    pub id: String,
    pub href: String,
    pub path: String,
    pub media_type: String,
}

#[derive(Debug, Clone)]
pub struct EditInfo {
    pub path: PathBuf,
    pub opf_path: String,
    pub base: String,
    pub title: String,
    pub author: String,
    pub lang: String,
    pub cover: Option<CoverInfo>,
    pub cover_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct CoverUpload {
    pub bytes: Vec<u8>,
    pub ext: CoverExt,
}

#[derive(Debug, Clone)]
pub struct Edits {
    pub title: String,
    pub author: String,
    pub lang: String,
    pub cover: Option<CoverUpload>,
}

#[derive(Debug, Clone)]
pub struct SaveResult {
    pub backup: PathBuf,
}

/// Lee metadatos, OPF y portada sin modificar nada.
pub fn read_info(path: impl AsRef<Path>) -> Result<EditInfo, String> {
    let path = path.as_ref().to_path_buf();
    let file = File::open(&path).map_err(|e| format!("no pude leer {}: {e}", path.display()))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("no es un EPUB válido: {e}"))?;

    let container = read_entry(&mut archive, "META-INF/container.xml")
        .ok_or_else(|| "falta META-INF/container.xml".to_string())?;
    let container_text = String::from_utf8_lossy(&container);
    let container_clean = crate::xhtml::sanitize(&container_text);
    let container_doc = roxmltree::Document::parse(&container_clean)
        .map_err(|e| format!("container.xml inválido: {e}"))?;
    let opf_path = container_doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "rootfile")
        .and_then(|n| n.attribute("full-path"))
        .ok_or_else(|| "container.xml sin rootfile".to_string())?
        .to_string();

    let opf_bytes =
        read_entry(&mut archive, &opf_path).ok_or_else(|| format!("falta el OPF {opf_path}"))?;
    let opf_text = String::from_utf8_lossy(&opf_bytes).to_string();
    let opf_clean = crate::xhtml::sanitize(&opf_text);
    let doc = roxmltree::Document::parse(&opf_clean).map_err(|e| format!("OPF inválido: {e}"))?;
    let base = dir_of(&opf_path);

    let metadata = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "metadata");
    let mut title = String::new();
    let mut author = String::new();
    let mut lang = String::new();
    let mut cover_meta_id: Option<String> = None;
    if let Some(metadata) = metadata {
        for node in metadata.descendants().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "title" if title.is_empty() => title = node_text(node),
                "creator" if author.is_empty() => author = node_text(node),
                "language" if lang.is_empty() => lang = node_text(node),
                "meta"
                    if node
                        .attribute("name")
                        .is_some_and(|name| name.eq_ignore_ascii_case("cover")) =>
                {
                    cover_meta_id = node
                        .attribute("content")
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string);
                }
                _ => {}
            }
        }
    }

    let mut manifest: Vec<(String, String, String, String)> = Vec::new();
    for item in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "item")
    {
        let Some(id) = item.attribute("id") else {
            continue;
        };
        manifest.push((
            id.to_string(),
            item.attribute("href").unwrap_or_default().to_string(),
            item.attribute("media-type").unwrap_or_default().to_string(),
            item.attribute("properties").unwrap_or_default().to_string(),
        ));
    }

    let cover_item = manifest
        .iter()
        .find(|(_, _, _, props)| props.split_whitespace().any(|prop| prop == "cover-image"))
        .or_else(|| {
            cover_meta_id
                .as_ref()
                .and_then(|id| manifest.iter().find(|(item_id, _, _, _)| item_id == id))
        })
        .cloned();

    let cover = cover_item.map(|(id, href, media_type, _)| {
        let path = resolve(&base, &href);
        CoverInfo {
            id,
            href,
            path,
            media_type,
        }
    });
    let cover_bytes = cover
        .as_ref()
        .and_then(|cover| read_entry(&mut archive, &cover.path));

    Ok(EditInfo {
        path,
        opf_path,
        base,
        title,
        author,
        lang,
        cover,
        cover_bytes,
    })
}

/// Guarda metadatos/portada, valida el EPUB resultante y reemplaza el original.
/// Crea un backup en `backup_dir` antes de tocar el archivo.
pub fn save(
    path: impl AsRef<Path>,
    edits: &Edits,
    backup_dir: impl AsRef<Path>,
) -> Result<SaveResult, String> {
    let path = path.as_ref().to_path_buf();
    let info = read_info(&path)?;
    let title = edits.title.trim();
    if title.is_empty() {
        return Err("El título no puede quedar vacío.".into());
    }
    let author = edits.author.trim();
    let lang = edits.lang.trim();

    let file = File::open(&path).map_err(|e| format!("no pude leer {}: {e}", path.display()))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("no es un EPUB válido: {e}"))?;
    let opf_bytes = read_entry(&mut archive, &info.opf_path)
        .ok_or_else(|| format!("falta el OPF {}", info.opf_path))?;
    let mut opf = String::from_utf8_lossy(&opf_bytes).to_string();

    set_dc_element(&mut opf, "dc:title", title)?;
    set_dc_element(&mut opf, "dc:creator", author)?;
    set_dc_element(&mut opf, "dc:language", lang)?;

    let cover_plan = if let Some(upload) = &edits.cover {
        let (id, href, cover_path) = if let Some(cover) = &info.cover {
            let keep_href = cover
                .href
                .to_ascii_lowercase()
                .ends_with(&format!(".{}", upload.ext.extension()));
            let href = if keep_href {
                cover.href.clone()
            } else {
                format!("cover.{}", upload.ext.extension())
            };
            let cover_path = resolve(&info.base, &href);
            (cover.id.clone(), href, cover_path)
        } else {
            let href = format!("cover.{}", upload.ext.extension());
            let cover_path = resolve(&info.base, &href);
            ("candil-cover-image".to_string(), href, cover_path)
        };
        set_manifest_cover(
            &mut opf,
            &id,
            &href,
            upload.ext.media_type(),
            info.cover.as_ref().map(|cover| cover.id.as_str()),
        )?;
        set_meta_cover(&mut opf, &id)?;
        Some((cover_path, upload.bytes.clone()))
    } else {
        None
    };

    let temp = temporary_path(&path);
    let result = write_epub(
        &mut archive,
        &temp,
        &info.opf_path,
        opf.as_bytes(),
        cover_plan.as_ref(),
    )
    .and_then(|()| {
        let mut book = Book::open(&temp).map_err(|e| format!("El EPUB editado no abre: {e}"))?;
        if cover_plan.is_some() && book.cover_bytes().is_none() {
            return Err("La portada no quedó accesible en el EPUB editado.".into());
        }
        Ok(())
    })
    .and_then(|()| {
        let backup = backup_path(backup_dir.as_ref(), &path)?;
        fs::copy(&path, &backup)
            .map_err(|e| format!("no pude crear el backup {}: {e}", backup.display()))?;
        replace_file(&temp, &path, &backup)?;
        Ok(SaveResult { backup })
    });

    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn write_epub(
    archive: &mut ZipArchive<File>,
    temp: &Path,
    opf_path: &str,
    opf: &[u8],
    cover: Option<&(String, Vec<u8>)>,
) -> Result<(), String> {
    let out = File::create(temp).map_err(|e| format!("no pude crear {}: {e}", temp.display()))?;
    let mut writer = ZipWriter::new(out);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    writer
        .start_file("mimetype", stored)
        .map_err(|e| format!("no pude escribir mimetype: {e}"))?;
    writer
        .write_all(b"application/epub+zip")
        .map_err(|e| format!("no pude escribir mimetype: {e}"))?;

    let mut cover_written = false;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("no pude leer una entrada del EPUB: {e}"))?;
        let name = entry
            .name()
            .map_err(|e| format!("nombre inválido en el EPUB: {e}"))?
            .to_string();
        if name == "mimetype" || entry.is_dir() {
            continue;
        }
        if name == opf_path {
            write_entry(
                &mut writer,
                &name,
                entry.compression(),
                entry.unix_mode(),
                opf,
            )?;
            continue;
        }
        if let Some((cover_path, bytes)) = cover {
            if name == *cover_path {
                write_entry(
                    &mut writer,
                    &name,
                    CompressionMethod::Stored,
                    entry.unix_mode(),
                    bytes,
                )?;
                cover_written = true;
                continue;
            }
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| format!("no pude leer {name}: {e}"))?;
        write_entry(
            &mut writer,
            &name,
            entry.compression(),
            entry.unix_mode(),
            &bytes,
        )?;
    }

    if let Some((cover_path, bytes)) = cover {
        if !cover_written {
            write_entry(
                &mut writer,
                cover_path,
                CompressionMethod::Stored,
                None,
                bytes,
            )?;
        }
    }

    writer
        .finish()
        .map_err(|e| format!("no pude cerrar el EPUB: {e}"))?;
    Ok(())
}

fn write_entry(
    writer: &mut ZipWriter<File>,
    name: &str,
    compression: CompressionMethod,
    unix_mode: Option<u32>,
    bytes: &[u8],
) -> Result<(), String> {
    let mut options = SimpleFileOptions::default().compression_method(compression);
    if let Some(mode) = unix_mode {
        options = options.unix_permissions(mode);
    }
    writer
        .start_file(name, options)
        .map_err(|e| format!("no pude escribir {name}: {e}"))?;
    writer
        .write_all(bytes)
        .map_err(|e| format!("no pude escribir {name}: {e}"))
}

fn set_dc_element(opf: &mut String, tag: &str, value: &str) -> Result<(), String> {
    let open = format!("<{tag}");
    let value = escape_xml(value);
    if let Some(start) = opf.find(&open) {
        let Some(tag_end_rel) = opf[start..].find('>') else {
            return Err(format!("OPF mal formado: {tag} sin cierre de etiqueta"));
        };
        let tag_end = start + tag_end_rel;
        if opf[start..=tag_end].trim_end().ends_with("/>") {
            let replacement = opf[start..=tag_end].replacen("/>", &format!(">{value}</{tag}>"), 1);
            opf.replace_range(start..=tag_end, &replacement);
            return Ok(());
        }
        let close = format!("</{tag}>");
        let Some(close_rel) = opf[tag_end..].find(&close) else {
            return Err(format!("OPF mal formado: falta {close}"));
        };
        let close_start = tag_end + close_rel;
        opf.replace_range(tag_end + 1..close_start, &value);
        return Ok(());
    }
    let Some(pos) = opf.find("</metadata>") else {
        return Err("OPF sin </metadata>".into());
    };
    opf.insert_str(pos, &format!("<{tag}>{value}</{tag}>"));
    Ok(())
}

fn set_manifest_cover(
    opf: &mut String,
    id: &str,
    href: &str,
    media_type: &str,
    existing_id: Option<&str>,
) -> Result<(), String> {
    let item = format!(
        "<item id=\"{}\" href=\"{}\" media-type=\"{}\" properties=\"cover-image\"/>",
        escape_attr(id),
        escape_attr(href),
        escape_attr(media_type)
    );
    if let Some(existing_id) = existing_id {
        let needle = format!("id=\"{}\"", escape_attr(existing_id));
        let mut search = 0;
        while let Some(rel) = opf[search..].find("<item") {
            let start = search + rel;
            let Some(end_rel) = opf[start..].find('>') else {
                break;
            };
            let end = start + end_rel;
            if opf[start..=end].contains(&needle) {
                opf.replace_range(start..=end, &item);
                return Ok(());
            }
            search = end + 1;
        }
    }
    let Some(pos) = opf.find("</manifest>") else {
        return Err("OPF sin </manifest>".into());
    };
    opf.insert_str(pos, &item);
    Ok(())
}

fn set_meta_cover(opf: &mut String, id: &str) -> Result<(), String> {
    let meta = format!("<meta name=\"cover\" content=\"{}\"/>", escape_attr(id));
    let mut search = 0;
    while let Some(rel) = opf[search..].find("<meta") {
        let start = search + rel;
        let Some(end_rel) = opf[start..].find('>') else {
            break;
        };
        let end = start + end_rel;
        if opf[start..=end].contains("name=\"cover\"") {
            opf.replace_range(start..=end, &meta);
            return Ok(());
        }
        search = end + 1;
    }
    let Some(pos) = opf.find("</metadata>") else {
        return Err("OPF sin </metadata>".into());
    };
    opf.insert_str(pos, &meta);
    Ok(())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_attr(value: &str) -> String {
    escape_xml(value).replace('\'', "&apos;")
}

fn node_text(node: roxmltree::Node<'_, '_>) -> String {
    node.descendants()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect::<String>()
        .trim()
        .to_string()
}

fn temporary_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "libro.epub".into());
    path.with_file_name(format!(".{name}.candil-tmp-{}-{nanos}", std::process::id()))
}

fn backup_path(dir: &Path, path: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(dir).map_err(|e| format!("no pude crear {}: {e}", dir.display()))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let stem = path
        .file_stem()
        .map(|stem| {
            stem.to_string_lossy()
                .chars()
                .map(|ch| {
                    if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                        ch
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
        })
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "libro".into());
    Ok(dir.join(format!("{stem}-{stamp}.epub")))
}

#[cfg(unix)]
fn replace_file(temp: &Path, path: &Path, _backup: &Path) -> Result<(), String> {
    fs::rename(temp, path).map_err(|e| format!("no pude reemplazar el EPUB: {e}"))
}

#[cfg(windows)]
fn replace_file(temp: &Path, path: &Path, backup: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| format!("no pude reemplazar el EPUB: {e}"))?;
    if let Err(error) = fs::rename(temp, path) {
        let _ = fs::copy(backup, path);
        return Err(format!("no pude reemplazar el EPUB: {error}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn temp_dir(tag: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("candil-edit-{tag}-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    fn png() -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(80, 120, image::Rgba([180, 80, 40, 255]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();
        bytes
    }

    #[test]
    fn agrega_portada_y_metadata_a_un_epub_sin_portada() {
        let dir = temp_dir("add");
        let path = dir.join("edge.epub");
        fs::copy(fixture("edge.epub"), &path).unwrap();
        let backup = dir.join("backups");
        let edits = Edits {
            title: "Título corregido".into(),
            author: "Autora & Otra".into(),
            lang: "en".into(),
            cover: Some(CoverUpload {
                bytes: png(),
                ext: CoverExt::Png,
            }),
        };
        let result = save(&path, &edits, &backup).unwrap();
        assert!(result.backup.is_file());

        let mut book = Book::open(&path).unwrap();
        assert_eq!(book.title, "Título corregido");
        assert_eq!(book.author, "Autora & Otra");
        assert_eq!(book.lang, "en");
        assert!(book.cover_bytes().is_some());

        let file = File::open(&path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();
        let first = archive.by_index(0).unwrap();
        assert_eq!(first.name().unwrap(), "mimetype");
        assert_eq!(first.compression(), CompressionMethod::Stored);
        drop(first);
        let opf =
            String::from_utf8(read_entry(&mut archive, "OEBPS/content.opf").unwrap()).unwrap();
        assert!(opf.contains("properties=\"cover-image\""));
        assert!(opf.contains("name=\"cover\""));
        assert!(opf.contains("dc:title>Título corregido"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn reemplaza_portada_existente() {
        let dir = temp_dir("replace");
        let path = dir.join("mini.epub");
        fs::copy(fixture("mini.epub"), &path).unwrap();
        let before = read_info(&path).unwrap().cover_bytes.unwrap();
        let edits = Edits {
            title: "Libro de prueba".into(),
            author: "Autora Ficticia".into(),
            lang: "es".into(),
            cover: Some(CoverUpload {
                bytes: png(),
                ext: CoverExt::Png,
            }),
        };
        save(&path, &edits, dir.join("backups")).unwrap();
        let after = read_info(&path).unwrap().cover_bytes.unwrap();
        assert_ne!(before, after);
        assert!(Book::open(&path).unwrap().cover_bytes().is_some());
        let _ = fs::remove_dir_all(dir);
    }
}
