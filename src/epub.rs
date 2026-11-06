//! Lectura del contenedor EPUB: zip → container.xml → OPF → spine + índice.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use image::GenericImageView;
use zip::ZipArchive;

use crate::xhtml::{self, Block};

#[derive(Debug, Clone)]
pub struct TocEntry {
    pub title: String,
    pub spine: usize,
    pub anchor: Option<String>,
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
    cover_candidates: Vec<String>,
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
        let mut cover_meta: Option<String> = None;
        if let Some(md) = doc
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "metadata")
        {
            for n in md.descendants().filter(|n| n.is_element()) {
                match n.tag_name().name() {
                    "title" if title.is_empty() => title = node_text(n),
                    "creator" if author.is_empty() => author = node_text(n),
                    "language" if lang.is_empty() => lang = node_text(n),
                    "meta" => {
                        let name = n.attribute("name").unwrap_or("");
                        let prop = n.attribute("property").unwrap_or("");
                        let is_cover = name.eq_ignore_ascii_case("cover")
                            || prop.eq_ignore_ascii_case("cover")
                            || prop.eq_ignore_ascii_case("cover-image");
                        if is_cover && cover_meta.is_none() {
                            cover_meta = n
                                .attribute("content")
                                .or_else(|| n.attribute("href"))
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty());
                        }
                    }
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

        // Portada declarada: `properties="cover-image"` (EPUB3) y
        // `meta name="cover"` (EPUB2/Calibre). El fallback a la primera imagen
        // del spine se resuelve en `cover_bytes()`.
        let mut cover_candidates: Vec<String> = Vec::new();
        if let Some(item) = manifest
            .values()
            .find(|m| m.props.split_whitespace().any(|p| p == "cover-image"))
        {
            cover_candidates.push(item.path.clone());
        }
        if let Some(meta) = cover_meta {
            if let Some(item) = manifest.get(&meta) {
                cover_candidates.push(item.path.clone());
            } else {
                let path = resolve(&base, &meta);
                if !path.is_empty() {
                    cover_candidates.push(path);
                }
            }
        }
        let mut seen_covers = HashSet::new();
        cover_candidates.retain(|path| seen_covers.insert(path.clone()));

        // Spine. Se descarta el documento `nav` (el índice) cuando hay otros
        // capítulos: algunos EPUB lo meten en el spine y se ve como una página
        // en blanco. Si el libro solo tiene ese documento, se conserva.
        let spine_node = doc
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "spine");
        let mut spine_items: Vec<(String, bool)> = Vec::new();
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
                        let is_nav = item.props.split_whitespace().any(|p| p == "nav");
                        spine_items.push((item.path.clone(), is_nav));
                    }
                }
            }
        }
        let has_reading_content = spine_items.iter().any(|(_, is_nav)| !is_nav);
        let spine: Vec<String> = spine_items
            .into_iter()
            .filter(|(_, is_nav)| !has_reading_content || !is_nav)
            .map(|(path, _)| path)
            .collect();
        if spine.is_empty() {
            return Err("el libro no declara capítulos legibles".into());
        }

        // Índice: primero nav.xhtml (EPUB3), después toc.ncx (EPUB2)
        let mut toc: Vec<(String, String, Option<String>)> = Vec::new();
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

        let entries = build_toc(toc, &spine);

        Ok(Self {
            path,
            title,
            author,
            lang,
            spine,
            toc: entries,
            archive,
            titles: HashMap::new(),
            cover_candidates,
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

    /// Resuelve un enlace interno a `(spine, ancla)`. Devuelve `None` para
    /// enlaces externos o destinos que no están en el spine.
    pub fn link_target(&self, href: &str, current: usize) -> Option<(usize, Option<String>)> {
        let href = href.trim();
        if href.is_empty() {
            return None;
        }
        let lower = href.to_ascii_lowercase();
        if lower.starts_with("http:")
            || lower.starts_with("https:")
            || lower.starts_with("mailto:")
            || lower.starts_with("tel:")
            || lower.starts_with("data:")
            || lower.starts_with("javascript:")
            || href.starts_with("//")
        {
            return None;
        }
        let (path, anchor) = split_fragment(href);
        let spine = if path.is_empty() {
            current
        } else {
            let base = self
                .spine
                .get(current)
                .map(|p| dir_of(p))
                .unwrap_or_default();
            let target = resolve(&base, path);
            spine_index(&self.spine, &target)?
        };
        if spine >= self.spine.len() {
            return None;
        }
        Some((spine, anchor))
    }

    /// Primer candidato de portada que decodifica y tiene aspecto de portada.
    ///
    /// Orden: declarada en el OPF; si no hay, la primera imagen de los primeros
    /// documentos del spine (incluye `<svg><image>`).
    pub fn cover_bytes(&mut self) -> Option<Vec<u8>> {
        let declared = self.cover_candidates.clone();
        if let Some(bytes) = self.valid_cover(&declared) {
            return Some(bytes);
        }

        let mut fallback: Vec<String> = Vec::new();
        for i in 0..self.spine.len().min(4) {
            let blocks = self.chapter_blocks(i);
            let mut found = false;
            for block in blocks
                .iter()
                .filter(|b| b.kind == crate::xhtml::Kind::Image)
            {
                if let Some(src) = &block.src {
                    if !fallback.contains(src) {
                        fallback.push(src.clone());
                    }
                    found = true;
                }
            }
            if found {
                break;
            }
        }
        self.valid_cover(&fallback)
    }

    fn valid_cover(&mut self, paths: &[String]) -> Option<Vec<u8>> {
        for path in paths {
            let Some(bytes) = self.resource(path) else {
                continue;
            };
            let Ok(img) = image::load_from_memory(&bytes) else {
                continue;
            };
            let (w, h) = img.dimensions();
            if w == 0 || h == 0 {
                continue;
            }
            // Un mapa/banner apaisado no es una portada.
            if (w as f32 / h as f32) > 2.0 {
                continue;
            }
            return Some(bytes);
        }
        None
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

fn split_fragment(href: &str) -> (&str, Option<String>) {
    match href.split_once('#') {
        Some((path, fragment)) if !fragment.is_empty() => (path, Some(percent_decode(fragment))),
        Some((path, _)) => (path, None),
        None => (href, None),
    }
}

fn build_toc(raw: Vec<(String, String, Option<String>)>, spine: &[String]) -> Vec<TocEntry> {
    let mut entries: Vec<TocEntry> = Vec::new();
    for (title, href, anchor) in raw {
        let target = resolve("", &href);
        let Some(spine_index) = spine_index(spine, &target) else {
            continue;
        };
        let duplicate = entries
            .last()
            .map(|e| e.spine == spine_index && e.anchor.as_deref() == anchor.as_deref())
            .unwrap_or(false);
        if !duplicate {
            entries.push(TocEntry {
                title,
                spine: spine_index,
                anchor,
            });
        }
    }
    entries
}

fn parse_nav(xml: &str, base: &str) -> Vec<(String, String, Option<String>)> {
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
        let (path, anchor) = split_fragment(href);
        let path = resolve(base, path);
        if !title.is_empty() && !path.is_empty() {
            out.push((title, path, anchor));
        }
    }
    out
}

fn parse_ncx(xml: &str, base: &str) -> Vec<(String, String, Option<String>)> {
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
        let (path, anchor) = split_fragment(src);
        let path = resolve(base, path);
        if !label.trim().is_empty() && !path.is_empty() {
            out.push((label, path, anchor));
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

    #[test]
    fn nav_preserves_fragments() {
        let nav = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="Text/c.xhtml#a">A</a></li><li><a href="Text/c.xhtml#b">B</a></li></ol></nav></body></html>"#;
        let out = parse_nav(nav, "OEBPS");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].1, "OEBPS/Text/c.xhtml");
        assert_eq!(out[0].2.as_deref(), Some("a"));
        assert_eq!(out[1].2.as_deref(), Some("b"));
    }

    #[test]
    fn toc_dedupes_consecutive_spine_and_anchor() {
        let spine = vec!["c.xhtml".to_string()];
        let raw = vec![
            (
                "A".to_string(),
                "c.xhtml".to_string(),
                Some("a".to_string()),
            ),
            (
                "A repetido".to_string(),
                "c.xhtml".to_string(),
                Some("a".to_string()),
            ),
            (
                "B".to_string(),
                "c.xhtml".to_string(),
                Some("b".to_string()),
            ),
            ("Sin ancla".to_string(), "c.xhtml".to_string(), None),
        ];
        let entries = build_toc(raw, &spine);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].anchor.as_deref(), Some("a"));
        assert_eq!(entries[1].anchor.as_deref(), Some("b"));
        assert_eq!(entries[2].anchor, None);
    }

    #[test]
    fn resolve_internal_links() {
        let book = Book::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/mini.epub"
        ))
        .unwrap();
        assert_eq!(
            book.link_target("cap3.xhtml#final", 0),
            Some((2, Some("final".to_string())))
        );
        assert_eq!(
            book.link_target("#nota", 1),
            Some((1, Some("nota".to_string())))
        );
        assert_eq!(
            book.link_target("../Text/cap2.xhtml", 0)
                .map(|(spine, _)| spine),
            Some(1)
        );
        assert!(book.link_target("https://example.com", 0).is_none());
    }
}
