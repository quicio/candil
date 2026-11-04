//! Modelo intermedio entre XHTML y egui.
//!
//! Un capítulo es una secuencia plana de bloques; cada bloque es una secuencia
//! de *runs* con estilo inline. Sin DOM, sin CSS: solo lo que un libro necesita.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Para,
    Heading(u8),
    Quote,
    Bullet,
    Numbered(u32),
    Rule,
    Pre,
    Caption,
    Image,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub small: bool,
    pub link: bool,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub kind: Kind,
    pub runs: Vec<Run>,
    /// Ruta dentro del zip (solo para imágenes).
    pub src: Option<String>,
    pub alt: String,
}

impl Block {
    pub fn text(&self) -> String {
        self.runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<String>()
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

const XHTML_NS: &str = "http://www.w3.org/1999/xhtml";

#[derive(Clone)]
struct Ctx {
    kind: Kind,
    bold: bool,
    italic: bool,
    small: bool,
    link: bool,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            kind: Kind::Para,
            bold: false,
            italic: false,
            small: false,
            link: false,
        }
    }
}

/// Parsea un documento XHTML y devuelve sus bloques.
pub fn parse(xhtml: &str) -> Vec<Block> {
    let cleaned = sanitize(xhtml);
    let doc = match roxmltree::Document::parse(&cleaned) {
        Ok(d) => d,
        // Muchos EPUB reales traen `text/html` que no es XML válido.
        Err(_) => return parse_lenient(&cleaned),
    };

    let body = doc
        .descendants()
        .find(|n| n.is_element() && n.tag_name().name() == "body");

    let mut out = Vec::new();
    let ctx = Ctx::default();
    match body {
        Some(body) => {
            let mut buf = Vec::new();
            visit(body, &mut out, &ctx, &mut buf);
            flush(&mut out, &ctx, &mut buf);
        }
        None => {
            let mut buf = Vec::new();
            visit_node(doc.root_element(), &mut out, &ctx, &mut buf);
            flush(&mut out, &ctx, &mut buf);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Parser tolerante (sin DOM): para documentos que no son XML bien formado.
// ---------------------------------------------------------------------------

fn parse_lenient(src: &str) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let mut buf: Vec<Run> = Vec::new();
    let mut bold = 0i32;
    let mut italic = 0i32;
    let mut small = 0i32;
    let mut link = 0i32;
    let mut kinds: Vec<Kind> = vec![Kind::Para];
    let mut lists: Vec<(bool, u32)> = Vec::new();

    let ctx = |kinds: &Vec<Kind>, b: i32, i: i32, s: i32, l: i32| Ctx {
        kind: *kinds.last().unwrap_or(&Kind::Para),
        bold: b > 0,
        italic: i > 0,
        small: s > 0,
        link: l > 0,
    };

    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < src.len() {
        let tail = &src[i..];
        if tail.starts_with("<!--") {
            match tail[4..].find("-->") {
                Some(e) => i += 4 + e + 3,
                None => break,
            }
            continue;
        }
        if tail.starts_with("<![CDATA[") {
            match tail[9..].find("]]>") {
                Some(e) => {
                    let c = ctx(&kinds, bold, italic, small, link);
                    push_text(&mut buf, &tail[9..9 + e], &c);
                    i += 9 + e + 3;
                }
                None => break,
            }
            continue;
        }
        if tail.starts_with("<!") || tail.starts_with("<?") {
            match tail.find('>') {
                Some(e) => i += e + 1,
                None => break,
            }
            continue;
        }
        if bytes[i] == b'<' {
            let Some(end) = find_tag_end(src, i) else {
                break;
            };
            let inner = &src[i + 1..end];
            i = end + 1;

            let closing = inner.starts_with('/');
            let body = if closing { &inner[1..] } else { inner };
            let name: String = body
                .trim_start()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == ':' || *c == '-')
                .collect::<String>()
                .to_ascii_lowercase();

            if name.is_empty() {
                let c = ctx(&kinds, bold, italic, small, link);
                push_text(&mut buf, "<", &c);
                continue;
            }

            let blockish = matches!(
                name.as_str(),
                "p" | "div"
                    | "section"
                    | "article"
                    | "header"
                    | "footer"
                    | "aside"
                    | "main"
                    | "body"
                    | "html"
                    | "figure"
                    | "figcaption"
                    | "tr"
                    | "td"
                    | "dd"
                    | "dt"
                    | "blockquote"
                    | "ul"
                    | "ol"
                    | "li"
                    | "pre"
            ) || is_heading(&name);

            if closing {
                if blockish {
                    let c = ctx(&kinds, bold, italic, small, link);
                    flush(&mut out, &c, &mut buf);
                    if kinds.len() > 1 {
                        kinds.pop();
                    }
                    if name == "ul" || name == "ol" {
                        lists.pop();
                    }
                    if name == "blockquote" {
                        italic = (italic - 1).max(0);
                    }
                }
                match name.as_str() {
                    "b" | "strong" => bold = (bold - 1).max(0),
                    "i" | "em" | "cite" => italic = (italic - 1).max(0),
                    "small" | "sup" | "sub" => small = (small - 1).max(0),
                    "a" => link = (link - 1).max(0),
                    _ => {}
                }
                continue;
            }

            // Apertura
            if blockish {
                let c = ctx(&kinds, bold, italic, small, link);
                flush(&mut out, &c, &mut buf);
            }

            match name.as_str() {
                "br" => push_break(&mut buf),
                "hr" => {
                    let c = ctx(&kinds, bold, italic, small, link);
                    flush(&mut out, &c, &mut buf);
                    out.push(Block {
                        kind: Kind::Rule,
                        runs: Vec::new(),
                        src: None,
                        alt: String::new(),
                    });
                }
                "img" => {
                    let c = ctx(&kinds, bold, italic, small, link);
                    flush(&mut out, &c, &mut buf);
                    out.push(Block {
                        kind: Kind::Image,
                        runs: Vec::new(),
                        src: attr(body, "src"),
                        alt: attr(body, "alt").unwrap_or_default(),
                    });
                }
                "ul" => {
                    lists.push((false, 0));
                    kinds.push(Kind::Bullet);
                }
                "ol" => {
                    lists.push((true, 0));
                    kinds.push(Kind::Para);
                }
                "li" => {
                    let kind = match lists.last_mut() {
                        Some((true, n)) => {
                            *n += 1;
                            Kind::Numbered(*n)
                        }
                        _ => Kind::Bullet,
                    };
                    kinds.push(kind);
                }
                "blockquote" => {
                    kinds.push(Kind::Quote);
                    italic += 1;
                }
                "pre" => kinds.push(Kind::Pre),
                "figcaption" => kinds.push(Kind::Caption),
                "head" | "title" | "script" | "style" => {}
                other => {
                    if let Some(level) = heading_level(other) {
                        kinds.push(Kind::Heading(level));
                        bold += 1;
                    } else if blockish {
                        kinds.push(Kind::Para);
                    }
                }
            }

            match name.as_str() {
                "b" | "strong" => bold += 1,
                "i" | "em" | "cite" => italic += 1,
                "small" | "sup" | "sub" => small += 1,
                "a" => link += 1,
                _ => {}
            }

            // Saltar contenido no visible (head, scripts, estilos).
            if matches!(name.as_str(), "script" | "style" | "head") {
                if let Some(pos) = src[i..].to_lowercase().find(&format!("</{name}")) {
                    i += pos;
                }
                continue;
            }
        }

        let next = src[i..].find('<').map(|e| i + e).unwrap_or(src.len());
        let c = ctx(&kinds, bold, italic, small, link);
        push_text(&mut buf, &src[i..next], &c);
        i = next;
    }

    let c = ctx(&kinds, bold, italic, small, link);
    flush(&mut out, &c, &mut buf);
    out
}

fn is_heading(name: &str) -> bool {
    heading_level(name).is_some()
}

fn heading_level(name: &str) -> Option<u8> {
    match name {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

fn find_tag_end(src: &str, start: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut j = start + 1;
    let mut quote: u8 = 0;
    while j < b.len() {
        let c = b[j];
        if quote != 0 {
            if c == quote {
                quote = 0;
            }
        } else if c == b'"' || c == b'\'' {
            quote = c;
        } else if c == b'>' {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Lee un atributo de una etiqueta de apertura.
pub fn attr(tag: &str, name: &str) -> Option<String> {
    let mut prev_ok = true;
    for (idx, _) in tag.char_indices() {
        if !prev_ok {
            prev_ok = tag[idx..].starts_with(char::is_whitespace);
            continue;
        }
        if !tag[idx..].starts_with(name) {
            prev_ok = false;
            continue;
        }
        let after = tag[idx + name.len()..].trim_start();
        let Some(after) = after.strip_prefix('=') else {
            prev_ok = false;
            continue;
        };
        let after = after.trim_start();
        let quote = after.chars().next()?;
        if quote == '"' || quote == '\'' {
            let body = &after[1..];
            let end = body.find(quote)?;
            return Some(body[..end].to_string());
        }
        let end = after
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(after.len());
        return Some(after[..end].to_string());
    }
    None
}

fn local<'a, 'i>(n: roxmltree::Node<'a, 'i>) -> String {
    n.tag_name().name().to_ascii_lowercase()
}

fn is_xhtml_ish(n: roxmltree::Node) -> bool {
    match n.tag_name().namespace() {
        None => true,
        Some(ns) => ns == XHTML_NS || ns.ends_with("xhtml"),
    }
}

fn visit(node: roxmltree::Node, out: &mut Vec<Block>, ctx: &Ctx, buf: &mut Vec<Run>) {
    for child in node.children() {
        visit_node(child, out, ctx, buf);
    }
}

fn visit_node(child: roxmltree::Node, out: &mut Vec<Block>, ctx: &Ctx, buf: &mut Vec<Run>) {
    if child.is_text() {
        if let Some(t) = child.text() {
            push_text(buf, t, ctx);
        }
        return;
    }
    if !child.is_element() || !is_xhtml_ish(child) {
        return;
    }

    let tag = local(child);
    match tag.as_str() {
        "head" | "title" | "script" | "style" | "meta" | "link" => {}
        "body" | "html" => {
            visit(child, out, ctx, buf);
        }

        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            flush(out, ctx, buf);
            let level = tag[1..].parse::<u8>().unwrap_or(2);
            let mut c = ctx.clone();
            c.kind = Kind::Heading(level);
            c.bold = true;
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "p" | "div" | "section" | "article" | "header" | "footer" | "figcaption" => {
            flush(out, ctx, buf);
            let mut c = ctx.clone();
            if tag == "figcaption" {
                c.kind = Kind::Caption;
            } else if ctx.kind != Kind::Quote && ctx.kind != Kind::Bullet {
                c.kind = Kind::Para;
            }
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "blockquote" => {
            flush(out, ctx, buf);
            let mut c = ctx.clone();
            c.kind = Kind::Quote;
            c.italic = true;
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "ul" => {
            flush(out, ctx, buf);
            let mut c = ctx.clone();
            c.kind = Kind::Bullet;
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "ol" => {
            flush(out, ctx, buf);
            let mut n = 1u32;
            for li in child.children().filter(|n| n.is_element()) {
                if local(li) != "li" {
                    continue;
                }
                let mut c = ctx.clone();
                c.kind = Kind::Numbered(n);
                visit(li, out, &c, buf);
                flush(out, &c, buf);
                n += 1;
            }
        }

        "li" => {
            // <li> suelto (fuera de ul/ol): viñeta
            let mut c = ctx.clone();
            if !matches!(c.kind, Kind::Numbered(_)) {
                c.kind = Kind::Bullet;
            }
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "pre" => {
            flush(out, ctx, buf);
            let mut c = ctx.clone();
            c.kind = Kind::Pre;
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "hr" => {
            flush(out, ctx, buf);
            out.push(Block {
                kind: Kind::Rule,
                runs: Vec::new(),
                src: None,
                alt: String::new(),
            });
        }

        "img" => {
            flush(out, ctx, buf);
            out.push(Block {
                kind: Kind::Image,
                runs: Vec::new(),
                src: child.attribute("src").map(|s| s.to_string()),
                alt: child.attribute("alt").unwrap_or("").to_string(),
            });
        }

        "br" => push_break(buf),

        "em" | "i" | "cite" => {
            let mut c = ctx.clone();
            c.italic = true;
            visit(child, out, &c, buf);
        }

        "strong" | "b" => {
            let mut c = ctx.clone();
            c.bold = true;
            visit(child, out, &c, buf);
        }

        "small" | "sup" | "sub" => {
            let mut c = ctx.clone();
            c.small = true;
            visit(child, out, &c, buf);
        }

        "a" => {
            let mut c = ctx.clone();
            c.link = true;
            visit(child, out, &c, buf);
        }

        _ => visit(child, out, ctx, buf),
    }
}

fn push_text(buf: &mut Vec<Run>, raw: &str, ctx: &Ctx) {
    let text = normalize_ws(raw);
    if text.is_empty() {
        return;
    }
    // Une con el run anterior si comparte estilo.
    if let Some(last) = buf.last_mut() {
        if last.bold == ctx.bold
            && last.italic == ctx.italic
            && last.small == ctx.small
            && last.link == ctx.link
        {
            last.text.push_str(&text);
            return;
        }
    }
    buf.push(Run {
        text,
        bold: ctx.bold,
        italic: ctx.italic,
        small: ctx.small,
        link: ctx.link,
    });
}

fn flush(out: &mut Vec<Block>, ctx: &Ctx, buf: &mut Vec<Run>) {
    if buf.is_empty() {
        return;
    }
    let mut runs = std::mem::take(buf);
    // Recorta espacios en los extremos del bloque, conserva \n internos.
    if let Some(first) = runs.first_mut() {
        first.text = first.text.trim_start().to_string();
    }
    if let Some(last) = runs.last_mut() {
        last.text = last.text.trim_end().to_string();
    }
    runs.retain(|r| !r.text.is_empty());

    if runs.iter().all(|r| r.text.trim().is_empty()) {
        return;
    }
    out.push(Block {
        kind: ctx.kind,
        runs,
        src: None,
        alt: String::new(),
    });
}

/// Colapsa todo espacio en blanco del fuente (incluidos saltos de línea) a un
/// espacio simple: en XHTML el salto solo significa separación, nunca párrafo.
fn normalize_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = true;
    for ch in s.chars() {
        let is_space = ch.is_whitespace();
        if is_space {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

fn push_break(buf: &mut Vec<Run>) {
    match buf.last_mut() {
        Some(last) => last.text.push('\n'),
        None => buf.push(Run {
            text: "\n".into(),
            ..Default::default()
        }),
    }
}

// ---------------------------------------------------------------------------
// Saneamiento: los EPUB reales traen DOCTYPE con entidades y XML sucio.
// ---------------------------------------------------------------------------

pub fn sanitize(src: &str) -> String {
    let mut s = strip_doctype(src);
    s = replace_entities(&s);
    // Espacio no separable: egui no corta ahí, pero tampoco queremos NBSP raros.
    s.replace('\u{a0}', " ")
}

fn strip_doctype(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(idx) = rest.find("<!DOCTYPE") {
        out.push_str(&rest[..idx]);
        let after = &rest[idx..];
        // Busca el cierre '>' respetando corchetes internos del DTD.
        let mut depth = 0i32;
        let mut end = None;
        for (i, ch) in after.char_indices() {
            match ch {
                '[' => depth += 1,
                ']' => depth -= 1,
                '>' if depth <= 0 => {
                    end = Some(i + 1);
                    break;
                }
                _ => {}
            }
        }
        match end {
            Some(e) => rest = &after[e..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

fn replace_entities(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if ch != '&' {
            out.push(ch);
            continue;
        }
        // Busca el ';' de cierre en un margen razonable.
        let tail = &src[i..];
        match tail.find(';').filter(|&e| e <= 12) {
            Some(e) => {
                let name = &tail[1..e];
                let rep = if name.starts_with('#') {
                    decode_numeric(name)
                } else {
                    named(name)
                };
                match rep {
                    Some(txt) => {
                        out.push_str(&txt);
                        // Avanza el iterador hasta el ';'.
                        for _ in 0..(e) {
                            chars.next();
                        }
                    }
                    None => out.push('&'),
                }
            }
            None => out.push('&'),
        }
    }
    out
}

fn decode_numeric(name: &str) -> Option<String> {
    let body = &name[1..];
    let code = if let Some(hex) = body.strip_prefix('x').or_else(|| body.strip_prefix('X')) {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        body.parse::<u32>().ok()?
    };
    char::from_u32(code).map(|c| c.to_string())
}

fn named(name: &str) -> Option<String> {
    // Se emiten referencias numéricas, no caracteres literales: un `&quot;`
    // reemplazado por `"` dentro de un atributo rompe el XML.
    let s = match name {
        "amp" => "&#38;",
        "lt" => "&#60;",
        "gt" => "&#62;",
        "quot" => "&#34;",
        "apos" => "&#39;",
        "nbsp" => " ",
        "mdash" => "\u{2014}",
        "ndash" => "\u{2013}",
        "hellip" => "\u{2026}",
        "lsquo" => "\u{2018}",
        "rsquo" => "\u{2019}",
        "ldquo" => "\u{201c}",
        "rdquo" => "\u{201d}",
        "laquo" => "\u{ab}",
        "raquo" => "\u{bb}",
        "iexcl" => "\u{a1}",
        "iquest" => "\u{bf}",
        "deg" => "\u{b0}",
        "middot" => "\u{b7}",
        "bull" => "\u{2022}",
        "trade" => "\u{2122}",
        "copy" => "\u{a9}",
        "reg" => "\u{ae}",
        "shy" => "",
        _ => return None,
    };
    Some(s.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_blocks() {
        let src = r#"<html><body><h1>Título</h1>
            <p>Hola <em>mundo</em>.</p><hr/><blockquote>Cita</blockquote></body></html>"#;
        let b = parse(src);
        assert!(matches!(b[0].kind, Kind::Heading(1)));
        assert_eq!(b[0].text(), "Título");
        assert_eq!(b[1].text(), "Hola mundo.");
        assert!(b[1]
            .runs
            .iter()
            .any(|r| r.italic && r.text.contains("mundo")));
        assert_eq!(b[2].kind, Kind::Rule);
        assert_eq!(b[3].kind, Kind::Quote);
    }

    #[test]
    fn collapses_source_whitespace() {
        assert_eq!(parse("<p>una\n   dos\t tres</p>")[0].text(), "una dos tres");
    }

    #[test]
    fn br_is_hard_break() {
        assert_eq!(parse("<p>uno<br/>dos</p>")[0].text(), "uno\ndos");
    }

    #[test]
    fn named_entities_decode() {
        let t = parse("<p>a&nbsp;b &mdash; c &amp; d</p>")[0].text();
        assert!(t.contains('—') && t.contains('&'), "{t}");
    }

    #[test]
    fn quotes_inside_attributes_survive() {
        // Calibre escribe &quot; dentro de atributos: no debe romper el XML.
        let src = r#"<meta name="x" content="{&quot;a&quot;: &quot;&quot;}"/><p>ok</p>"#;
        assert_eq!(parse(src)[0].text(), "ok");
    }

    #[test]
    fn lenient_fallback_handles_broken_html() {
        // HTML que no es XML: atributo sin comillas.
        let src = r#"<html><body><h2>Cap</h2><p class=malformado>efectos: texto &amp; mas</p><p>otro</p></body></html>"#;
        let b = parse(src);
        assert!(b.len() >= 3, "{b:?}");
        assert!(b.iter().any(|x| x.text().contains("otro")));
        assert!(b.iter().any(|x| matches!(x.kind, Kind::Heading(2))));
    }

    #[test]
    fn ordered_lists_number() {
        let b = parse("<ol><li>uno</li><li>dos</li></ol>");
        assert!(matches!(b[0].kind, Kind::Numbered(1)), "{b:?}");
        assert!(matches!(b[1].kind, Kind::Numbered(2)), "{b:?}");
    }

    #[test]
    fn image_src_is_extracted() {
        let b = parse(r#"<p>x</p><img src="img/a.png" alt="fig"/>"#);
        let img = b.iter().find(|x| x.kind == Kind::Image).unwrap();
        assert_eq!(img.src.as_deref(), Some("img/a.png"));
        assert_eq!(img.alt, "fig");
    }

    #[test]
    fn does_not_panic_on_garbage() {
        for junk in [
            "",
            "<",
            "<<<>>>",
            "<p>",
            "&",
            "<p>a &b c</p>",
            "<!DOCTYPE html>",
        ] {
            let _ = parse(junk);
        }
    }

    #[test]
    fn strips_doctype_and_script() {
        let src = r#"<!DOCTYPE html SYSTEM "about:legacy-compat"><html><head><script>var x = '<p>no</p>';</script></head><body><p>sí</p></body></html>"#;
        let b = parse(src);
        assert_eq!(b.len(), 1, "{b:?}");
        assert_eq!(b[0].text(), "sí");
    }
}
