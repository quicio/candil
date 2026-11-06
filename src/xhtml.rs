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
    /// Destino del enlace (`#ancla` o ruta relativa), si es un `<a>`.
    pub href: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub kind: Kind,
    pub runs: Vec<Run>,
    /// Ruta dentro del zip (solo para imágenes).
    pub src: Option<String>,
    pub alt: String,
    /// `id` del elemento que originó el bloque (ancla del índice).
    pub anchor: Option<String>,
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
const SVG_NS: &str = "http://www.w3.org/2000/svg";

#[derive(Clone)]
struct Ctx {
    kind: Kind,
    bold: bool,
    italic: bool,
    small: bool,
    link: bool,
    href: Option<String>,
    anchor: Option<String>,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            kind: Kind::Para,
            bold: false,
            italic: false,
            small: false,
            link: false,
            href: None,
            anchor: None,
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
    let mut anchors: Vec<Option<String>> = vec![None];
    let mut current_href: Option<String> = None;
    let mut lists: Vec<(bool, u32)> = Vec::new();

    let ctx = |kinds: &Vec<Kind>,
               b: i32,
               i: i32,
               s: i32,
               l: i32,
               anchor: Option<String>,
               href: Option<String>| Ctx {
        kind: *kinds.last().unwrap_or(&Kind::Para),
        bold: b > 0,
        italic: i > 0,
        small: s > 0,
        link: l > 0,
        href,
        anchor,
    };

    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < src.len() {
        let tail = &src[i..];
        if let Some(rest) = tail.strip_prefix("<!--") {
            match rest.find("-->") {
                Some(e) => i += 4 + e + 3,
                None => break,
            }
            continue;
        }
        if let Some(rest) = tail.strip_prefix("<![CDATA[") {
            match rest.find("]]>") {
                Some(e) => {
                    let c = ctx(
                        &kinds,
                        bold,
                        italic,
                        small,
                        link,
                        anchors.last().cloned().flatten(),
                        current_href.clone(),
                    );
                    push_text(&mut buf, &rest[..e], &c);
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
                let c = ctx(
                    &kinds,
                    bold,
                    italic,
                    small,
                    link,
                    anchors.last().cloned().flatten(),
                    current_href.clone(),
                );
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
                    | "table"
                    | "thead"
                    | "tbody"
                    | "tfoot"
                    | "tr"
                    | "td"
                    | "th"
                    | "caption"
                    | "dl"
                    | "dd"
                    | "dt"
                    | "svg"
                    | "blockquote"
                    | "ul"
                    | "ol"
                    | "li"
                    | "pre"
            ) || is_heading(&name);

            if closing {
                if blockish {
                    let c = ctx(
                        &kinds,
                        bold,
                        italic,
                        small,
                        link,
                        anchors.last().cloned().flatten(),
                        current_href.clone(),
                    );
                    flush(&mut out, &c, &mut buf);
                    if kinds.len() > 1 {
                        kinds.pop();
                    }
                    if anchors.len() > 1 {
                        anchors.pop();
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
                    "a" => {
                        link = (link - 1).max(0);
                        if link == 0 {
                            current_href = None;
                        }
                    }
                    "th" | "dt" => bold = (bold - 1).max(0),
                    _ => {}
                }
                continue;
            }

            // Apertura
            if blockish {
                let c = ctx(
                    &kinds,
                    bold,
                    italic,
                    small,
                    link,
                    anchors.last().cloned().flatten(),
                    current_href.clone(),
                );
                flush(&mut out, &c, &mut buf);
                let inherited = anchors.last().cloned().flatten();
                let own = attr(body, "id").filter(|id| !id.trim().is_empty());
                anchors.push(own.or(inherited));
            } else if let Some(id) = attr(body, "id").filter(|id| !id.trim().is_empty()) {
                if let Some(last) = anchors.last_mut() {
                    *last = Some(id);
                }
            }

            match name.as_str() {
                "br" => push_break(&mut buf),
                "hr" => {
                    let c = ctx(
                        &kinds,
                        bold,
                        italic,
                        small,
                        link,
                        anchors.last().cloned().flatten(),
                        current_href.clone(),
                    );
                    flush(&mut out, &c, &mut buf);
                    out.push(Block {
                        kind: Kind::Rule,
                        runs: Vec::new(),
                        src: None,
                        alt: String::new(),
                        anchor: c.anchor.clone(),
                    });
                }
                "img" | "image" => {
                    let c = ctx(
                        &kinds,
                        bold,
                        italic,
                        small,
                        link,
                        anchors.last().cloned().flatten(),
                        current_href.clone(),
                    );
                    flush(&mut out, &c, &mut buf);
                    let src = if name.as_str() == "img" {
                        attr(body, "src")
                    } else {
                        attr(body, "xlink:href").or_else(|| attr(body, "href"))
                    };
                    out.push(Block {
                        kind: Kind::Image,
                        runs: Vec::new(),
                        src,
                        alt: attr(body, "alt")
                            .or_else(|| attr(body, "aria-label"))
                            .unwrap_or_default(),
                        anchor: c.anchor.clone(),
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
                "figcaption" | "caption" => kinds.push(Kind::Caption),
                "th" | "dt" => {
                    kinds.push(Kind::Para);
                    bold += 1;
                }
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
                "a" => {
                    link += 1;
                    current_href = attr(body, "href");
                }
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
        let c = ctx(
            &kinds,
            bold,
            italic,
            small,
            link,
            anchors.last().cloned().flatten(),
            current_href.clone(),
        );
        push_text(&mut buf, &src[i..next], &c);
        i = next;
    }

    let c = ctx(
        &kinds,
        bold,
        italic,
        small,
        link,
        anchors.last().cloned().flatten(),
        current_href.clone(),
    );
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

fn is_svg_ish(n: roxmltree::Node) -> bool {
    n.tag_name().namespace() == Some(SVG_NS)
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
    if !child.is_element() || (!is_xhtml_ish(child) && !is_svg_ish(child)) {
        return;
    }

    let tag = local(child);
    let mut local_ctx = ctx.clone();
    if let Some(id) = child.attribute("id") {
        if !id.trim().is_empty() {
            local_ctx.anchor = Some(id.to_string());
        }
    }
    let ctx = &local_ctx;
    let before = out.len();
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

        // Tablas y listas de definición: no hay render tabular, pero cada
        // celda/entrada es un bloque propio (antes se pegaban todas).
        "table" | "thead" | "tbody" | "tfoot" | "tr" => {
            flush(out, ctx, buf);
            visit(child, out, ctx, buf);
            flush(out, ctx, buf);
        }

        "td" | "th" => {
            flush(out, ctx, buf);
            let mut c = ctx.clone();
            c.kind = Kind::Para;
            if tag == "th" {
                c.bold = true;
            }
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "caption" => {
            flush(out, ctx, buf);
            let mut c = ctx.clone();
            c.kind = Kind::Caption;
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        "dl" | "dt" | "dd" => {
            flush(out, ctx, buf);
            let mut c = ctx.clone();
            c.kind = Kind::Para;
            if tag == "dt" {
                c.bold = true;
            }
            visit(child, out, &c, buf);
            flush(out, &c, buf);
        }

        // Portadas y mapas de EPUB reales envuelven un raster en
        // <svg><image xlink:href="..."/></svg>. No se rasterizan vectores.
        "svg" => {
            flush(out, ctx, buf);
            let image = child
                .descendants()
                .find(|n| n.is_element() && n.tag_name().name().eq_ignore_ascii_case("image"));
            if let Some(image) = image {
                let src = image
                    .attributes()
                    .find(|a| a.name().eq_ignore_ascii_case("href"))
                    .map(|a| a.value().to_string())
                    .filter(|s| !s.trim().is_empty());
                if let Some(src) = src {
                    out.push(Block {
                        kind: Kind::Image,
                        runs: Vec::new(),
                        src: Some(src),
                        alt: image
                            .attribute("aria-label")
                            .or_else(|| image.attribute("title"))
                            .unwrap_or("")
                            .to_string(),
                        anchor: ctx.anchor.clone(),
                    });
                }
            }
        }

        "hr" => {
            flush(out, ctx, buf);
            out.push(Block {
                kind: Kind::Rule,
                runs: Vec::new(),
                src: None,
                alt: String::new(),
                anchor: ctx.anchor.clone(),
            });
        }

        "img" => {
            flush(out, ctx, buf);
            out.push(Block {
                kind: Kind::Image,
                runs: Vec::new(),
                src: child.attribute("src").map(|s| s.to_string()),
                alt: child.attribute("alt").unwrap_or("").to_string(),
                anchor: ctx.anchor.clone(),
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
            c.href = child.attribute("href").map(|href| href.to_string());
            visit(child, out, &c, buf);
        }

        _ => visit(child, out, ctx, buf),
    }

    // Un `id` en un elemento sin bloque propio (típico: `<a id="6"></a>`
    // antes de un título) se conserva como bloque-ancla invisible. El lector
    // lo usa para el mapa de saltos y no lo dibuja.
    if let Some(id) = child.attribute("id") {
        if !id.trim().is_empty()
            && !matches!(
                tag.as_str(),
                "head" | "title" | "script" | "style" | "meta" | "link"
            )
            && !out[before..]
                .iter()
                .any(|b| b.anchor.as_deref() == Some(id))
        {
            out.push(Block {
                kind: Kind::Para,
                runs: Vec::new(),
                src: None,
                alt: String::new(),
                anchor: Some(id.to_string()),
            });
        }
    }
}

fn push_text(buf: &mut Vec<Run>, raw: &str, ctx: &Ctx) {
    let is_pre = ctx.kind == Kind::Pre;
    // `<pre>` conserva saltos e indentación: son contenido, no separación.
    let mut text = if is_pre {
        raw.to_string()
    } else {
        normalize_ws(raw)
    };
    if text.is_empty() {
        return;
    }
    // Une con el run anterior si comparte estilo.
    if let Some(last) = buf.last_mut() {
        // Un espacio de frontera nunca se duplica, aunque cambie el estilo.
        if !is_pre && last.text.ends_with(' ') && text.starts_with(' ') {
            text.remove(0);
            if text.is_empty() {
                return;
            }
        }
        if last.bold == ctx.bold
            && last.italic == ctx.italic
            && last.small == ctx.small
            && last.link == ctx.link
            && last.href == ctx.href
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
        href: ctx.href.clone(),
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
        anchor: ctx.anchor.clone(),
    });
}

/// Colapsa el whitespace interno a un espacio simple, pero **conserva** un
/// espacio en los bordes: entre dos runs inline un espacio del fuente no puede
/// desaparecer. `flush` recorta los extremos del bloque.
fn normalize_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !in_space {
                out.push(' ');
                in_space = true;
            }
        } else {
            out.push(ch);
            in_space = false;
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
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let mut s = strip_doctype(src);
    s = replace_entities(&s);
    // Espacio no separable: egui no corta ahí, pero tampoco queremos NBSP raros.
    // El soft hyphen no sirve sin hyphenation y ensucia búsquedas/copiar.
    s.replace('\u{a0}', " ").replace('\u{ad}', "")
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

/// Entidades HTML que aparecen en EPUB reales. No es la lista completa de
/// HTML5 (~2.200): cubre Latin-1, griego, matemáticas y puntuación comunes.
/// Lo desconocido queda literal para que sea visible y depurable.
const NAMED_ENTITIES: &[(&str, &str)] = &[
    // XML / básicas. Se emiten como referencias numéricas: un `&quot;`
    // reemplazado por `"` dentro de un atributo rompe el XML.
    ("amp", "&#38;"),
    ("lt", "&#60;"),
    ("gt", "&#62;"),
    ("quot", "&#34;"),
    ("apos", "&#39;"),
    ("nbsp", " "),
    ("shy", ""),
    ("ensp", "\u{2002}"),
    ("emsp", "\u{2003}"),
    ("thinsp", "\u{2009}"),
    ("zwnj", ""),
    ("zwj", ""),
    // Puntuación y símbolos.
    ("ndash", "\u{2013}"),
    ("mdash", "\u{2014}"),
    ("hellip", "\u{2026}"),
    ("lsquo", "\u{2018}"),
    ("rsquo", "\u{2019}"),
    ("sbquo", "\u{201a}"),
    ("ldquo", "\u{201c}"),
    ("rdquo", "\u{201d}"),
    ("bdquo", "\u{201e}"),
    ("laquo", "\u{ab}"),
    ("raquo", "\u{bb}"),
    ("lsaquo", "\u{2039}"),
    ("rsaquo", "\u{203a}"),
    ("dagger", "\u{2020}"),
    ("Dagger", "\u{2021}"),
    ("bull", "\u{2022}"),
    ("permil", "\u{2030}"),
    ("prime", "\u{2032}"),
    ("Prime", "\u{2033}"),
    ("oline", "\u{203e}"),
    ("frasl", "\u{2044}"),
    ("sect", "\u{a7}"),
    ("para", "\u{b6}"),
    ("middot", "\u{b7}"),
    ("copy", "\u{a9}"),
    ("reg", "\u{ae}"),
    ("trade", "\u{2122}"),
    ("euro", "\u{20ac}"),
    ("cent", "\u{a2}"),
    ("pound", "\u{a3}"),
    ("yen", "\u{a5}"),
    ("curren", "\u{a4}"),
    ("brvbar", "\u{a6}"),
    ("uml", "\u{a8}"),
    ("macr", "\u{af}"),
    ("acute", "\u{b4}"),
    ("cedil", "\u{b8}"),
    ("not", "\u{ac}"),
    ("plusmn", "\u{b1}"),
    ("times", "\u{d7}"),
    ("divide", "\u{f7}"),
    ("frac12", "\u{bd}"),
    ("frac14", "\u{bc}"),
    ("frac34", "\u{be}"),
    ("sup1", "\u{b9}"),
    ("sup2", "\u{b2}"),
    ("sup3", "\u{b3}"),
    ("micro", "\u{b5}"),
    ("deg", "\u{b0}"),
    ("iexcl", "\u{a1}"),
    ("iquest", "\u{bf}"),
    ("ordf", "\u{aa}"),
    ("ordm", "\u{ba}"),
    // Latin-1 y Latin Extended que aparecen en libros en español.
    ("aacute", "\u{e1}"),
    ("agrave", "\u{e0}"),
    ("acirc", "\u{e2}"),
    ("atilde", "\u{e3}"),
    ("auml", "\u{e4}"),
    ("aring", "\u{e5}"),
    ("aelig", "\u{e6}"),
    ("ccedil", "\u{e7}"),
    ("eacute", "\u{e9}"),
    ("egrave", "\u{e8}"),
    ("ecirc", "\u{ea}"),
    ("euml", "\u{eb}"),
    ("iacute", "\u{ed}"),
    ("igrave", "\u{ec}"),
    ("icirc", "\u{ee}"),
    ("iuml", "\u{ef}"),
    ("eth", "\u{f0}"),
    ("ntilde", "\u{f1}"),
    ("oacute", "\u{f3}"),
    ("ograve", "\u{f2}"),
    ("ocirc", "\u{f4}"),
    ("otilde", "\u{f5}"),
    ("ouml", "\u{f6}"),
    ("oslash", "\u{f8}"),
    ("uacute", "\u{fa}"),
    ("ugrave", "\u{f9}"),
    ("ucirc", "\u{fb}"),
    ("uuml", "\u{fc}"),
    ("yacute", "\u{fd}"),
    ("yuml", "\u{ff}"),
    ("thorn", "\u{fe}"),
    ("szlig", "\u{df}"),
    ("Aacute", "\u{c1}"),
    ("Agrave", "\u{c0}"),
    ("Acirc", "\u{c2}"),
    ("Atilde", "\u{c3}"),
    ("Auml", "\u{c4}"),
    ("Aring", "\u{c5}"),
    ("AElig", "\u{c6}"),
    ("Ccedil", "\u{c7}"),
    ("Eacute", "\u{c9}"),
    ("Egrave", "\u{c8}"),
    ("Ecirc", "\u{ca}"),
    ("Euml", "\u{cb}"),
    ("Iacute", "\u{cd}"),
    ("Igrave", "\u{cc}"),
    ("Icirc", "\u{ce}"),
    ("Iuml", "\u{cf}"),
    ("ETH", "\u{d0}"),
    ("Ntilde", "\u{d1}"),
    ("Oacute", "\u{d3}"),
    ("Ograve", "\u{d2}"),
    ("Ocirc", "\u{d4}"),
    ("Otilde", "\u{d5}"),
    ("Ouml", "\u{d6}"),
    ("Oslash", "\u{d8}"),
    ("Uacute", "\u{da}"),
    ("Ugrave", "\u{d9}"),
    ("Ucirc", "\u{db}"),
    ("Uuml", "\u{dc}"),
    ("Yacute", "\u{dd}"),
    ("Yuml", "\u{178}"),
    ("THORN", "\u{de}"),
    ("OElig", "\u{152}"),
    ("oelig", "\u{153}"),
    ("Scaron", "\u{160}"),
    ("scaron", "\u{161}"),
    ("fnof", "\u{192}"),
    ("circ", "\u{2c6}"),
    ("tilde", "\u{2dc}"),
    // Griego.
    ("alpha", "\u{3b1}"),
    ("beta", "\u{3b2}"),
    ("gamma", "\u{3b3}"),
    ("delta", "\u{3b4}"),
    ("epsilon", "\u{3b5}"),
    ("zeta", "\u{3b6}"),
    ("eta", "\u{3b7}"),
    ("theta", "\u{3b8}"),
    ("iota", "\u{3b9}"),
    ("kappa", "\u{3ba}"),
    ("lambda", "\u{3bb}"),
    ("mu", "\u{3bc}"),
    ("nu", "\u{3bd}"),
    ("xi", "\u{3be}"),
    ("omicron", "\u{3bf}"),
    ("pi", "\u{3c0}"),
    ("rho", "\u{3c1}"),
    ("sigmaf", "\u{3c2}"),
    ("sigma", "\u{3c3}"),
    ("tau", "\u{3c4}"),
    ("upsilon", "\u{3c5}"),
    ("phi", "\u{3c6}"),
    ("chi", "\u{3c7}"),
    ("psi", "\u{3c8}"),
    ("omega", "\u{3c9}"),
    ("Alpha", "\u{391}"),
    ("Beta", "\u{392}"),
    ("Gamma", "\u{393}"),
    ("Delta", "\u{394}"),
    ("Epsilon", "\u{395}"),
    ("Zeta", "\u{396}"),
    ("Eta", "\u{397}"),
    ("Theta", "\u{398}"),
    ("Iota", "\u{399}"),
    ("Kappa", "\u{39a}"),
    ("Lambda", "\u{39b}"),
    ("Mu", "\u{39c}"),
    ("Nu", "\u{39d}"),
    ("Xi", "\u{39e}"),
    ("Omicron", "\u{39f}"),
    ("Pi", "\u{3a0}"),
    ("Rho", "\u{3a1}"),
    ("Sigma", "\u{3a3}"),
    ("Tau", "\u{3a4}"),
    ("Upsilon", "\u{3a5}"),
    ("Phi", "\u{3a6}"),
    ("Chi", "\u{3a7}"),
    ("Psi", "\u{3a8}"),
    ("Omega", "\u{3a9}"),
    // Matemáticas.
    ("le", "\u{2264}"),
    ("ge", "\u{2265}"),
    ("ne", "\u{2260}"),
    ("equiv", "\u{2261}"),
    ("asymp", "\u{2248}"),
    ("infin", "\u{221e}"),
    ("sum", "\u{2211}"),
    ("prod", "\u{220f}"),
    ("radic", "\u{221a}"),
    ("int", "\u{222b}"),
    ("part", "\u{2202}"),
    ("nabla", "\u{2207}"),
    ("isin", "\u{2208}"),
    ("notin", "\u{2209}"),
    ("ni", "\u{220b}"),
    ("sub", "\u{2282}"),
    ("sup", "\u{2283}"),
    ("subset", "\u{2282}"),
    ("supset", "\u{2283}"),
    ("and", "\u{2227}"),
    ("or", "\u{2228}"),
    ("cap", "\u{2229}"),
    ("cup", "\u{222a}"),
    ("forall", "\u{2200}"),
    ("exist", "\u{2203}"),
    ("empty", "\u{2205}"),
    ("there4", "\u{2234}"),
    ("sim", "\u{223c}"),
    ("cong", "\u{2245}"),
    ("prop", "\u{221d}"),
    ("perp", "\u{22a5}"),
    ("ang", "\u{2220}"),
    ("sdot", "\u{22c5}"),
    ("lowast", "\u{2217}"),
    ("minus", "\u{2212}"),
    ("plus", "\u{2b}"),
    ("larr", "\u{2190}"),
    ("uarr", "\u{2191}"),
    ("rarr", "\u{2192}"),
    ("darr", "\u{2193}"),
    ("harr", "\u{2194}"),
    ("crarr", "\u{21b5}"),
    ("lArr", "\u{21d0}"),
    ("uArr", "\u{21d1}"),
    ("rArr", "\u{21d2}"),
    ("dArr", "\u{21d3}"),
    ("hArr", "\u{21d4}"),
];

fn named(name: &str) -> Option<String> {
    NAMED_ENTITIES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, value)| (*value).to_string())
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

    #[test]
    fn inline_boundaries_keep_a_single_space() {
        assert_eq!(
            parse("<p><b>Hola</b> <i>mundo</i></p>")[0].text(),
            "Hola mundo"
        );
        assert_eq!(parse("<p>Hola<b> mundo</b></p>")[0].text(), "Hola mundo");
        assert_eq!(parse("<p>a <em> b </em> c</p>")[0].text(), "a b c");
        // Sin espacio en el fuente no se inventa uno.
        assert_eq!(parse("<p>uno<em>dos</em>tres</p>")[0].text(), "unodostres");
    }

    #[test]
    fn latin1_greek_and_math_entities_decode() {
        let t = parse("<p>caf&eacute; ni&ntilde;o &le; &omega; &Phi;</p>")[0].text();
        assert_eq!(t, "caf\u{e9} ni\u{f1}o \u{2264} \u{3c9} \u{3a6}");
    }

    #[test]
    fn soft_hyphen_and_bom_are_removed() {
        assert_eq!(parse("\u{feff}<p>a&shy;b c\u{ad}d</p>")[0].text(), "ab cd");
    }

    #[test]
    fn pre_preserves_whitespace_and_newlines() {
        let b = parse("<pre>def f(x):\n    return x</pre>");
        assert_eq!(b[0].kind, Kind::Pre);
        assert_eq!(b[0].text(), "def f(x):\n    return x");
    }

    #[test]
    fn tables_and_definition_lists_do_not_glue() {
        let b = parse("<table><tr><td>celda uno</td><td>celda dos</td></tr></table><dl><dt>t\u{e9}rmino</dt><dd>definici\u{f3}n</dd></dl>");
        let texts: Vec<String> = b.iter().map(|x| x.text()).collect();
        assert!(texts.iter().any(|t| t == "celda uno"), "{texts:?}");
        assert!(texts.iter().any(|t| t == "celda dos"), "{texts:?}");
        assert!(texts.iter().any(|t| t == "t\u{e9}rmino"), "{texts:?}");
        assert!(texts.iter().any(|t| t == "definici\u{f3}n"), "{texts:?}");
        assert!(
            !texts.iter().any(|t| t.contains("celda unocelda")),
            "{texts:?}"
        );
    }

    #[test]
    fn table_header_and_caption_are_separate() {
        let b = parse(
            "<table><caption>Leyenda</caption><tr><th>Encabezado</th><td>dato</td></tr></table>",
        );
        assert!(b
            .iter()
            .any(|x| x.kind == Kind::Caption && x.text() == "Leyenda"));
        let th = b
            .iter()
            .find(|x| x.text() == "Encabezado")
            .expect("encabezado");
        assert!(th.runs.iter().all(|r| r.bold), "{:?}", th.runs);
        assert!(b.iter().any(|x| x.text() == "dato"));
    }

    #[test]
    fn svg_image_is_extracted_in_strict_and_lenient() {
        let strict = r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:xlink="http://www.w3.org/1999/xlink"><body><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2 2"><image xlink:href="img/a.png" width="2" height="2"/></svg></body></html>"#;
        let b = parse(strict);
        let img = b
            .iter()
            .find(|x| x.kind == Kind::Image)
            .expect("imagen svg");
        assert_eq!(img.src.as_deref(), Some("img/a.png"));

        let lenient = r#"<html><body><svg><image xlink:href="img/b.png"></svg></body></html>"#;
        let b = parse(lenient);
        let img = b
            .iter()
            .find(|x| x.kind == Kind::Image)
            .expect("imagen svg tolerante");
        assert_eq!(img.src.as_deref(), Some("img/b.png"));
    }

    #[test]
    fn anchors_are_captured_in_strict_and_lenient() {
        let strict = parse(r#"<div id="sec"><h2 id="cap">Cap</h2><p>Texto</p></div>"#);
        assert_eq!(strict[0].anchor.as_deref(), Some("cap"));
        assert_eq!(strict[1].anchor.as_deref(), Some("sec"));

        let lenient = parse("<html><body><h2 id=cap>Cap</h2><p>Texto</p></body></html>");
        assert_eq!(lenient[0].anchor.as_deref(), Some("cap"));

        // Un ancla vacía antes del texto no se pierde.
        let empty = parse(r#"<p><a id="marca"></a>Texto</p>"#);
        assert!(empty.iter().any(|b| b.anchor.as_deref() == Some("marca")));
    }

    #[test]
    fn link_href_is_captured() {
        let strict = parse(r##"<p>ver <a href="#nota">nota</a></p>"##);
        let run = strict[0]
            .runs
            .iter()
            .find(|run| run.href.is_some())
            .expect("href estricto");
        assert_eq!(run.href.as_deref(), Some("#nota"));
        assert!(run.link);

        let lenient = parse("<p>ver <a href=#n>nota</a></p>");
        let run = lenient[0]
            .runs
            .iter()
            .find(|run| run.href.is_some())
            .expect("href tolerante");
        assert_eq!(run.href.as_deref(), Some("#n"));
    }
}
