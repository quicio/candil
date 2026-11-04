//! Lectura del contenedor EPUB: zip → container.xml → OPF → spine + índice.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use crate::xhtml::{self, Block};

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub spine: usize,
}

#[derive(Debug, Clone, Default)]
struct ManifestItem {
    path: String,
    media_type: String,
    props: String,
}

pub struct Book {
    pub path: PathBuf,
    pub title: String,
    pub author: String,
    pub lang: String,
    pub spine: Vec<String>,
    pub toc: Vec<TocEntry>,
    archive: ZipArchive<File>,
    titles: HashMap<usize, String>,
}

impl Book {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let file =
            File::open(&path).map_err(|e| format!("no pude leer {}: {e}", path.display()))?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| format!("no es un EPUB válido: {e}"))?;

        let container = read_entry(&mut archive, "META-INF/container.xml")
            .ok_or_else(|| "falta META-INF/container.xml".to_string())?;
        let container_text = String::from_utf8_lossy(&container).to_string();
        let container_clean = xhtml::sanitize(&container_text);
        let opf_path = {
            let doc = roxmltree::Document::parse(&container_clean)
                .map_err(|e| format!("container.xml inválido: {e}"))?;
            doc.descendants()
                .find(|n| n.is_element() && n.tag_name().name() == "rootfile")
                .and_then(|n| n.attribute("full-path"))
                .ok_or_else(|| "container.xml sin rootfile".to_string())?
                .to_string()
        };

        let opf_bytes = read_entry(&mut archive, &opf_path)
            .ok_or_else(|| format!("falta el OPF {opf_path}"))?;
        let opf_text = String::from_utf8_lossy(&opf_bytes).to_string();
        let opf_clean = xhtml::sanitize(&opf_text);
        let doc =
            roxmltree::Document::parse(&opf_clean).map_err(|e| format!("OPF inválido: {e}"))?;

        let base = dir_of(&opf_path);

        let mut title = String::new();
        let mut author = String::new();
        let mut lang = String::new();
        if let Some(md) = doc
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "metadata")
        {
            for n in md.descendants().filter(|n| n.is_element()) {
                match n.tag_name().name() {
                    "title" if title.is_empty() => title = node_text(n),
                    "creator" if author.is_empty() => author = node_text(n),
                    "language" if lang.is_empty() => lang = node_text(n),
                    _ => {}
                }
            }
        }
        if title.trim().is_empty() {
            title = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Sin título".into());
        }

        // Manifest
        let mut manifest: HashMap<String, ManifestItem> = HashMap::new();
        for item in doc.descendants().filter(|n| {
            n.is_element() && n.tag_name().name() == "item" && n.attribute("id").is_some()
        }) {
            let id = item.attribute("id").unwrap_or_default().to_string();
            let href = item.attribute("href").unwrap_or_default();
            let media_type = item.attribute("media-type").unwrap_or_default().to_string();
            let props = item.attribute("properties").unwrap_or_default().to_string();
            manifest.insert(
                id,
                ManifestItem {
                    path: resolve(&base, href),
                    media_type,
                    props,
                },
            );
        }

        // Spine
        let spine_node = doc
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "spine");
        let mut spine = Vec::new();
        let mut toc_id = None;
        if let Some(sp) = spine_node {
            toc_id = sp.attribute("toc").map(|s| s.to_string());
            for iref in sp
                .children()
                .filter(|n| n.is_element() && n.tag_name().name() == "itemref")
            {
                let idref = iref.attribute("idref").unwrap_or_default();
                if let Some(item) = manifest.get(idref) {
                    if !item.path.is_empty() {
                        spine.push(item.path.clone());
                    }
                }
            }
        }
        if spine.is_empty() {
            return Err("el libro no declara capítulos legibles".into());
        }

        // Índice: primero nav.xhtml (EPUB3), después toc.ncx (EPUB2)
        let mut toc: Vec<(String, String)> = Vec::new();
        if let Some(nav) = manifest
            .values()
            .find(|m| m.props.split_whitespace().any(|p| p == "nav"))
        {
            if let Some(bytes) = read_entry(&mut archive, &nav.path) {
                let text = String::from_utf8_lossy(&bytes).to_string();
                toc = parse_nav(&text, &dir_of(&nav.path));
            }
        }
        if toc.is_empty() {
            if let Some(ncx_id) = toc_id {
                if let Some(ncx) = manifest.get(&ncx_id) {
                    if let Some(bytes) = read_entry(&mut archive, &ncx.path) {
                        let text = String::from_utf8_lossy(&bytes).to_string();
                        toc = parse_ncx(&text, &dir_of(&ncx.path));
                    }
                }
            }
        }
        if toc.is_empty() {
            // Fallback: cualquier ncx declarado en el manifest
            if let Some(ncx) = manifest
                .values()
                .find(|m| m.media_type == "application/x-dtbncx+xml")
            {
                if let Some(bytes) = read_entry(&mut archive, &ncx.path) {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    toc = parse_ncx(&text, &dir_of(&ncx.path));
                }
            }
        }

        let mut entries: Vec<TocEntry> = Vec::new();
        for (t, href) in toc {
            let target = resolve("", &href);
            if let Some(idx) = spine_index(&spine, &target) {
                if entries.last().map(|e| e.spine) != Some(idx) {
                    entries.push(TocEntry {
                        title: t,
                        spine: idx,
                    });
                }
            }
        }

        Ok(Self {
            path,
            title,
            author,
            lang,
            spine,
            toc: entries,
            archive,
            titles: HashMap::new(),
        })
    }

    pub fn chapter_count(&self) -> usize {
        self.spine.len()
    }

    /// Parsea un capítulo del spine (0-indexed).
    pub fn chapter_blocks(&mut self, index: usize) -> Vec<Block> {
        let Some(path) = self.spine.get(index).cloned() else {
            return Vec::new();
        };
        let Some(bytes) = read_entry(&mut self.archive, &path) else {
            return Vec::new();
        };
        let text = String::from_utf8_lossy(&bytes).to_string();
        let mut blocks = xhtml::parse(&text);
        let base = dir_of(&path);
        for b in &mut blocks {
            if b.kind == crate::xhtml::Kind::Image {
                if let Some(src) = &b.src {
                    b.src = Some(resolve(&base, src));
                }
            }
        }
        blocks
    }

    /// Título del capítulo: primer encabezado, o el del índice, o "Capítulo N".
    pub fn chapter_title(&mut self, index: usize) -> String {
        if let Some(t) = self.titles.get(&index) {
            return t.clone();
        }
        let from_toc = self
            .toc
            .iter()
            .find(|e| e.spine == index)
            .map(|e| e.title.clone());
        if let Some(t) = from_toc {
            self.titles.insert(index, t.clone());
            return t;
        }
        let blocks = self.chapter_blocks(index);
        let heading = blocks
            .iter()
            .find(|b| matches!(b.kind, crate::xhtml::Kind::Heading(_)))
            .map(|b| b.text())
            .filter(|t| !t.trim().is_empty());
        let t = heading.unwrap_or_else(|| format!("Capítulo {}", index + 1));
        self.titles.insert(index, t.clone());
        t
    }

    /// Bytes de un recurso interno (imágenes, css, fuentes).
    pub fn resource(&mut self, path: &str) -> Option<Vec<u8>> {
        read_entry(&mut self.archive, path)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn node_text(n: roxmltree::Node) -> String {
    n.descendants()
        .filter(|d| d.is_text())
        .filter_map(|d| d.text())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_entry(archive: &mut ZipArchive<File>, name: &str) -> Option<Vec<u8>> {
    let candidates = [name.to_string(), format!("./{name}"), format!("/{name}")];
    let mut hit = None;
    for cand in &candidates {
        if archive.by_name(cand).is_ok() {
            hit = Some(cand.clone());
            break;
        }
    }
    if hit.is_none() {
        // Fallback: coincidencia sin distinguir mayúsculas (EPUBs sucios).
        let lower = name.trim_start_matches("./").to_lowercase();
        let found = archive
            .file_names()
            .filter_map(|n| n.ok().map(|c| c.to_string()))
            .find(|n| n.trim_start_matches("./").to_lowercase() == lower);
        hit = found;
    }
    let name = hit?;
    let mut f = archive.by_name(&name).ok()?;
    let mut buf = Vec::with_capacity(f.size() as usize);
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

pub fn dir_of(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((d, _)) => d.to_string(),
        None => String::new(),
    }
}

/// Resuelve un href relativo contra un directorio base, dentro del zip.
pub fn resolve(base: &str, href: &str) -> String {
    let href = href.split('#').next().unwrap_or("");
    let href = percent_decode(href);
    let joined = if let Some(stripped) = href.strip_prefix('/') {
        stripped.to_string()
    } else if base.is_empty() {
        href
    } else {
        format!("{base}/{href}")
    };
    normalize(&joined)
}

fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn spine_index(spine: &[String], target: &str) -> Option<usize> {
    if let Some(i) = spine.iter().position(|s| s == target) {
        return Some(i);
    }
    let file = target.rsplit('/').next().unwrap_or(target);
    spine.iter().position(|s| {
        let f = s.rsplit('/').next().unwrap_or(s);
        f == file
    })
}

fn parse_nav(xml: &str, base: &str) -> Vec<(String, String)> {
    let cleaned = xhtml::sanitize(xml);
    let Ok(doc) = roxmltree::Document::parse(&cleaned) else {
        return Vec::new();
    };
    let navs: Vec<_> = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "nav")
        .collect();
    let chosen = navs
        .iter()
        .find(|n| {
            n.attributes()
                .any(|a| a.name() == "type" && a.value().split_whitespace().any(|t| t == "toc"))
        })
        .or_else(|| navs.first());
    let Some(nav) = chosen else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for a in nav
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "a")
    {
        let title = node_text(a);
        let Some(href) = a.attribute("href") else {
            continue;
        };
        let path = resolve(base, href);
        if !title.is_empty() && !path.is_empty() {
            out.push((title, path));
        }
    }
    out
}

fn parse_ncx(xml: &str, base: &str) -> Vec<(String, String)> {
    let cleaned = xhtml::sanitize(xml);
    let Ok(doc) = roxmltree::Document::parse(&cleaned) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for point in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "navPoint")
    {
        let label = point
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "navLabel")
            .map(node_text)
            .unwrap_or_default();
        let src = point
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "content")
            .and_then(|n| n.attribute("src"))
            .unwrap_or("");
        let path = resolve(base, src);
        if !label.trim().is_empty() && !path.is_empty() {
            out.push((label, path));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_paths() {
        assert_eq!(
            resolve("OEBPS/Text", "../Images/a.png"),
            "OEBPS/Images/a.png"
        );
        assert_eq!(resolve("", "./a/b.html#frag"), "a/b.html");
        assert_eq!(resolve("OEBPS", "/x/y.html"), "x/y.html");
        assert_eq!(resolve("a/b", "../../c.html"), "c.html");
    }

    #[test]
    fn decodes_percent_escapes() {
        assert_eq!(resolve("", "mi%20libro.html"), "mi libro.html");
        assert_eq!(percent_decode("a%2Fb"), "a/b");
        assert_eq!(percent_decode("sin-escapes"), "sin-escapes");
    }

    #[test]
    fn matches_spine_by_filename() {
        let spine = vec![
            "OEBPS/Text/c1.xhtml".to_string(),
            "OEBPS/c2.xhtml".to_string(),
        ];
        assert_eq!(spine_index(&spine, "OEBPS/Text/c1.xhtml"), Some(0));
        assert_eq!(spine_index(&spine, "Text/c1.xhtml"), Some(0));
        assert_eq!(spine_index(&spine, "nada.xhtml"), None);
    }

    #[test]
    fn dirname_of_entry() {
        assert_eq!(dir_of("OEBPS/Text/c1.xhtml"), "OEBPS/Text");
        assert_eq!(dir_of("c1.xhtml"), "");
    }
}
