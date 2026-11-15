//! Render local de una cita como imagen para compartir.
//!
//! La imagen se compone fuera de la pantalla a resolución real (1080 px de
//! ancho) con `image` para el fondo y `ab_glyph` para el texto. Así la vista
//! previa y el PNG final usan el mismo layout y el resultado no depende del
//! tamaño o del DPI de la ventana.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use ab_glyph::{Font, FontVec, GlyphId, PxScale, ScaleFont};
use image::{imageops, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::fonts;

const BASE_WIDTH: f32 = 1080.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareFormat {
    #[default]
    Story,
    Post,
    Square,
    Portrait,
    Landscape,
}

impl ShareFormat {
    pub fn size(self) -> (u32, u32) {
        match self {
            Self::Story => (1080, 1920),
            Self::Post => (1080, 1350),
            Self::Square => (1080, 1080),
            Self::Portrait => (1080, 1440),
            Self::Landscape => (1920, 1080),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Story => "Historia 9:16",
            Self::Post => "Publicación 4:5",
            Self::Square => "Cuadrado 1:1",
            Self::Portrait => "Retrato 3:4",
            Self::Landscape => "Horizontal 16:9",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Story,
            Self::Post,
            Self::Square,
            Self::Portrait,
            Self::Landscape,
        ]
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareBackground {
    #[default]
    Paper,
    Night,
    Cover,
    Solid,
    Image,
}

impl ShareBackground {
    pub fn label(self) -> &'static str {
        match self {
            Self::Paper => "Papel",
            Self::Night => "Noche",
            Self::Cover => "Portada",
            Self::Solid => "Color",
            Self::Image => "Imagen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShareSettings {
    pub format: ShareFormat,
    pub background: ShareBackground,
    pub solid_color: [u8; 3],
    pub image_path: Option<PathBuf>,
    pub show_book: bool,
    pub show_author: bool,
    pub show_chapter: bool,
    pub show_mark: bool,
}

impl Default for ShareSettings {
    fn default() -> Self {
        Self {
            format: ShareFormat::Story,
            background: ShareBackground::Paper,
            solid_color: [238, 230, 212],
            image_path: None,
            show_book: true,
            show_author: true,
            show_chapter: false,
            show_mark: true,
        }
    }
}

#[derive(Clone)]
pub struct ShareData {
    pub quote: String,
    pub book: String,
    pub author: String,
    pub chapter: Option<String>,
    pub cover: Option<Arc<RgbaImage>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    pub bg: [u8; 3],
    pub fg: [u8; 3],
    pub dim: [u8; 3],
    pub accent: [u8; 3],
}

impl ThemeColors {
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        Self {
            bg: [theme.bg.r(), theme.bg.g(), theme.bg.b()],
            fg: [theme.fg.r(), theme.fg.g(), theme.fg.b()],
            dim: [theme.dim.r(), theme.dim.g(), theme.dim.b()],
            accent: [theme.accent.r(), theme.accent.g(), theme.accent.b()],
        }
    }
}

pub struct Rendered {
    pub image: RgbaImage,
    pub truncated: bool,
    /// El fondo pedido no estaba disponible y se usó Papel.
    pub fallback: bool,
}

#[derive(Debug, Clone, Copy)]
struct Palette {
    bg: [u8; 3],
    bg_alt: [u8; 3],
    fg: [u8; 3],
    dim: [u8; 3],
    accent: [u8; 3],
}

#[derive(Clone, Copy)]
enum Face {
    Regular,
    Semibold,
}

struct Fonts {
    regular: FontVec,
    semibold: FontVec,
    fallbacks: Vec<FontVec>,
}

impl Fonts {
    fn load() -> Option<Self> {
        let parse = |bytes: &[u8]| FontVec::try_from_vec(bytes.to_vec()).ok();
        Some(Self {
            regular: parse(fonts::REGULAR)?,
            semibold: parse(fonts::SEMIBOLD)?,
            fallbacks: fonts::fallback_bytes()
                .into_iter()
                .filter_map(|bytes| parse(&bytes))
                .collect(),
        })
    }

    fn primary(&self, face: Face) -> &FontVec {
        match face {
            Face::Regular => &self.regular,
            Face::Semibold => &self.semibold,
        }
    }

    fn font_for(&self, face: Face, ch: char) -> &FontVec {
        let primary = self.primary(face);
        if primary.glyph_id(ch) != GlyphId(0) {
            return primary;
        }
        self.fallbacks
            .iter()
            .find(|font| font.glyph_id(ch) != GlyphId(0))
            .unwrap_or(primary)
    }

    fn measure(&self, face: Face, size: f32, text: &str) -> f32 {
        let mut width = 0.0;
        let mut previous: Option<(&FontVec, GlyphId)> = None;
        for ch in text.chars() {
            let font = self.font_for(face, ch);
            let scaled = font.as_scaled(size);
            let id = scaled.glyph_id(ch);
            if let Some((prev_font, prev_id)) = previous {
                if std::ptr::eq(prev_font, font) {
                    width += scaled.kern(prev_id, id);
                }
            }
            width += scaled.h_advance(id);
            previous = Some((font, id));
        }
        width
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_line(
        &self,
        image: &mut RgbaImage,
        face: Face,
        size: f32,
        color: [u8; 3],
        alpha: f32,
        center_x: f32,
        baseline: f32,
        text: &str,
        spacing: f32,
    ) {
        let chars = text.chars().count();
        let extra = spacing * chars.saturating_sub(1) as f32;
        let mut x = center_x - (self.measure(face, size, text) + extra) * 0.5;
        let mut previous: Option<(&FontVec, GlyphId)> = None;
        for ch in text.chars() {
            let font = self.font_for(face, ch);
            let scaled = font.as_scaled(size);
            let id = scaled.glyph_id(ch);
            let kern = previous
                .filter(|(prev_font, _)| std::ptr::eq(*prev_font, font))
                .map(|(_, prev_id)| scaled.kern(prev_id, id))
                .unwrap_or(0.0);
            x += kern;
            if !ch.is_control() && ch != ' ' {
                let glyph =
                    id.with_scale_and_position(PxScale::from(size), ab_glyph::point(x, baseline));
                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();
                    outlined.draw(|dx, dy, coverage| {
                        let px = bounds.min.x as i32 + dx as i32;
                        let py = bounds.min.y as i32 + dy as i32;
                        blend_at(image, px, py, color, coverage * alpha);
                    });
                }
            }
            x += scaled.h_advance(id) + spacing;
            previous = Some((font, id));
        }
    }
}

#[derive(Clone)]
struct QuoteLayout {
    paragraphs: Vec<Vec<String>>,
    line_height: f32,
    paragraph_gap: f32,
    height: f32,
}

impl QuoteLayout {
    fn recompute_height(&mut self) {
        let lines: usize = self.paragraphs.iter().map(Vec::len).sum();
        let gaps = self.paragraphs.len().saturating_sub(1) as f32;
        self.height = lines as f32 * self.line_height + gaps * self.paragraph_gap;
    }
}

/// Compone la imagen. `scale` permite generar la vista previa a menor tamaño;
/// el PNG final usa `scale = 1.0`.
pub fn render(
    data: &ShareData,
    settings: &ShareSettings,
    theme: &ThemeColors,
    custom_bg: Option<&RgbaImage>,
    scale: f32,
) -> Result<Rendered, String> {
    let fonts =
        Fonts::load().ok_or_else(|| "No pude cargar las fuentes para la imagen.".to_string())?;
    let scale = scale.clamp(0.05, 2.0);
    let (base_w, base_h) = settings.format.size();
    let width = (base_w as f32 * scale).round().max(64.0) as u32;
    let height = (base_h as f32 * scale).round().max(64.0) as u32;
    let u = width as f32 / BASE_WIDTH;

    let (mode, fallback) = match settings.background {
        ShareBackground::Cover if data.cover.is_some() => (ShareBackground::Cover, false),
        ShareBackground::Image if custom_bg.is_some() => (ShareBackground::Image, false),
        ShareBackground::Cover | ShareBackground::Image => (ShareBackground::Paper, true),
        other => (other, false),
    };
    let palette = palette_for(mode, settings, theme);
    let mut image = paint_background(mode, data, custom_bg, &palette, width, height);

    let footer = FooterMetrics::new(settings, data, height, u);
    let quote_top = height as f32 * 0.09;
    let quote_bottom = footer.top;
    let max_quote_height = (quote_bottom - quote_top).max(40.0 * u);
    let max_width = width as f32 - 232.0 * u;
    let max_size = 78.0 * u;
    let min_size = 42.0 * u;

    let (quote_size, mut layout, mut truncated) = fit_quote(
        &fonts,
        &data.quote,
        max_width,
        max_quote_height,
        min_size,
        max_size,
        u,
    );
    if layout.height > max_quote_height {
        truncated |= truncate_layout(&mut layout, &fonts, max_width, max_quote_height, quote_size);
    }

    let block_top = quote_top + ((quote_bottom - quote_top) - layout.height).max(0.0) * 0.5;
    let accent_y = block_top - 34.0 * u;
    if data.quote.trim().is_empty() {
        // Sin cita no hay adorno; el pie sigue mostrando el libro.
    } else if accent_y > quote_top - 20.0 * u {
        fill_rect(
            &mut image,
            ((width as f32 - 84.0 * u) * 0.5) as i32,
            accent_y as i32,
            (84.0 * u) as i32,
            (3.0 * u).max(1.0) as i32,
            palette.accent,
            0.9,
        );
    }

    let ascent = fonts.regular.as_scaled(quote_size).ascent();
    let mut y = block_top;
    for paragraph in &layout.paragraphs {
        for line in paragraph {
            if !line.is_empty() {
                fonts.draw_line(
                    &mut image,
                    Face::Regular,
                    quote_size,
                    palette.fg,
                    0.98,
                    width as f32 * 0.5,
                    y + ascent,
                    line,
                    0.0,
                );
            }
            y += layout.line_height;
        }
        y += layout.paragraph_gap;
    }
    footer.paint(&fonts, &mut image, &palette, settings, data, width, u);

    Ok(Rendered {
        image,
        truncated,
        fallback,
    })
}

struct FooterMetrics {
    top: f32,
    line_y: Option<f32>,
    title_y: Option<f32>,
    author_y: Option<f32>,
    chapter_y: Option<f32>,
    mark_y: Option<f32>,
}

impl FooterMetrics {
    fn new(settings: &ShareSettings, data: &ShareData, height: u32, u: f32) -> Self {
        let has_mark = settings.show_mark;
        let has_title = settings.show_book && !data.book.trim().is_empty();
        let has_author = settings.show_author && !data.author.trim().is_empty();
        let has_chapter = settings
            .show_chapter
            .then_some(data.chapter.as_deref())
            .flatten()
            .is_some_and(|chapter| !chapter.trim().is_empty());

        let mut y = height as f32 - 74.0 * u;
        let mark_y = has_mark.then_some(y);
        if has_chapter {
            y -= 38.0 * u;
        }
        let chapter_y = has_chapter.then_some(y);
        if has_author {
            y -= 40.0 * u;
        }
        let author_y = has_author.then_some(y);
        if has_title {
            y -= 50.0 * u;
        }
        let title_y = has_title.then_some(y);
        // Más aire entre la línea de adorno y el título: antes caía justo
        // sobre las ascendentes de la primera línea.
        let line_y = if has_title || has_author || has_chapter || has_mark {
            Some(y - 60.0 * u)
        } else {
            None
        };
        let top = line_y
            .map(|line| line - 40.0 * u)
            .unwrap_or(height as f32 - 64.0 * u);
        Self {
            top,
            line_y,
            title_y,
            author_y,
            chapter_y,
            mark_y,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint(
        &self,
        fonts: &Fonts,
        image: &mut RgbaImage,
        palette: &Palette,
        settings: &ShareSettings,
        data: &ShareData,
        width: u32,
        u: f32,
    ) {
        let center = width as f32 * 0.5;
        let max_width = width as f32 - 200.0 * u;
        if let Some(line_y) = self.line_y {
            fill_rect(
                image,
                ((width as f32 - 70.0 * u) * 0.5) as i32,
                line_y as i32,
                (70.0 * u) as i32,
                (2.0 * u).max(1.0) as i32,
                palette.accent,
                0.8,
            );
        }
        if let Some(y) = self.title_y {
            let title = fit_text(fonts, Face::Semibold, 40.0 * u, &data.book, max_width);
            fonts.draw_line(
                image,
                Face::Semibold,
                40.0 * u,
                palette.fg,
                1.0,
                center,
                y,
                &title,
                0.0,
            );
        }
        if let Some(y) = self.author_y {
            let author = fit_text(fonts, Face::Regular, 32.0 * u, &data.author, max_width);
            fonts.draw_line(
                image,
                Face::Regular,
                32.0 * u,
                palette.dim,
                1.0,
                center,
                y,
                &author,
                0.0,
            );
        }
        if let (Some(y), Some(chapter)) = (self.chapter_y, data.chapter.as_deref()) {
            let chapter = fit_text(fonts, Face::Regular, 26.0 * u, chapter, max_width);
            fonts.draw_line(
                image,
                Face::Regular,
                26.0 * u,
                palette.dim,
                0.95,
                center,
                y,
                &chapter,
                0.0,
            );
        }
        if settings.show_mark {
            if let Some(y) = self.mark_y {
                let mark = "CANDIL";
                fonts.draw_line(
                    image,
                    Face::Semibold,
                    21.0 * u,
                    palette.dim,
                    0.72,
                    center,
                    y,
                    mark,
                    6.0 * u,
                );
            }
        }
    }
}

fn fit_quote(
    fonts: &Fonts,
    quote: &str,
    max_width: f32,
    max_height: f32,
    min_size: f32,
    max_size: f32,
    u: f32,
) -> (f32, QuoteLayout, bool) {
    let min_size = min_size.max(8.0);
    let max_size = max_size.max(min_size);
    let step = (2.0 * u).max(1.0);
    let mut size = max_size;
    while size >= min_size {
        let layout = layout_quote(fonts, quote, max_width, size);
        if layout.height <= max_height {
            return (size, layout, false);
        }
        size -= step;
    }
    let layout = layout_quote(fonts, quote, max_width, min_size);
    (min_size, layout, false)
}

fn layout_quote(fonts: &Fonts, quote: &str, max_width: f32, size: f32) -> QuoteLayout {
    let line_height = size * 1.32;
    let paragraph_gap = size * 0.42;
    let paragraphs = quote
        .split('\n')
        .map(|paragraph| wrap_paragraph(fonts, paragraph, max_width, size))
        .collect::<Vec<_>>();
    let mut layout = QuoteLayout {
        paragraphs,
        line_height,
        paragraph_gap,
        height: 0.0,
    };
    layout.recompute_height();
    layout
}

fn wrap_paragraph(fonts: &Fonts, paragraph: &str, max_width: f32, size: f32) -> Vec<String> {
    let trimmed = paragraph.trim();
    if trimmed.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in trimmed.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if fonts.measure(Face::Regular, size, &candidate) <= max_width {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(current);
            current = String::new();
        }
        if fonts.measure(Face::Regular, size, word) <= max_width {
            current = word.to_string();
        } else {
            let mut chunks = break_word(fonts, word, max_width, size);
            if let Some(last) = chunks.pop() {
                lines.extend(chunks);
                current = last;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn break_word(fonts: &Fonts, word: &str, max_width: f32, size: f32) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for ch in word.chars() {
        let candidate = format!("{current}{ch}");
        if !current.is_empty() && fonts.measure(Face::Regular, size, &candidate) > max_width {
            chunks.push(current);
            current = ch.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn truncate_layout(
    layout: &mut QuoteLayout,
    fonts: &Fonts,
    max_width: f32,
    max_height: f32,
    size: f32,
) -> bool {
    let mut changed = false;
    while layout.height > max_height {
        if layout.paragraphs.iter().map(Vec::len).sum::<usize>() <= 1 {
            break;
        }
        let Some(paragraph) = layout.paragraphs.last_mut() else {
            break;
        };
        if paragraph.pop().is_some() {
            changed = true;
            if paragraph.is_empty() {
                layout.paragraphs.pop();
            }
            layout.recompute_height();
        } else {
            break;
        }
    }
    while let Some(paragraph) = layout
        .paragraphs
        .iter_mut()
        .rev()
        .find(|paragraph| paragraph.iter().any(|line| !line.is_empty()))
    {
        let Some(line) = paragraph.last_mut() else {
            break;
        };
        while !line.is_empty()
            && fonts.measure(Face::Regular, size, &format!("{line}…")) > max_width
        {
            let trimmed = line.trim_end();
            let cut = trimmed.rfind(' ').unwrap_or(0);
            *line = trimmed[..cut].trim_end().to_string();
            changed = true;
        }
        if line.is_empty() {
            paragraph.pop();
            changed = true;
            if paragraph.is_empty() {
                layout.paragraphs.pop();
            }
            continue;
        }
        *line = format!("{line}…");
        break;
    }
    layout.recompute_height();
    changed
}

fn fit_text(fonts: &Fonts, face: Face, size: f32, text: &str, max_width: f32) -> String {
    let text = text.trim();
    if fonts.measure(face, size, text) <= max_width {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{out}{ch}");
        if fonts.measure(face, size, &format!("{candidate}…")) > max_width {
            break;
        }
        out = candidate;
    }
    if out.is_empty() {
        "…".to_string()
    } else {
        format!("{}…", out.trim_end())
    }
}

fn palette_for(mode: ShareBackground, settings: &ShareSettings, theme: &ThemeColors) -> Palette {
    match mode {
        ShareBackground::Paper => Palette {
            bg: theme.bg,
            bg_alt: shift(theme.bg, if luma(theme.bg) > 0.5 { -0.045 } else { 0.045 }),
            fg: theme.fg,
            dim: theme.dim,
            accent: theme.accent,
        },
        ShareBackground::Night => Palette {
            bg: [18, 16, 14],
            bg_alt: [30, 26, 22],
            fg: [246, 240, 228],
            dim: [186, 176, 158],
            accent: ensure_visible(theme.accent, [18, 16, 14]),
        },
        ShareBackground::Cover | ShareBackground::Image => Palette {
            bg: [10, 9, 8],
            bg_alt: [22, 19, 16],
            fg: [248, 242, 230],
            dim: [214, 204, 186],
            accent: ensure_visible(theme.accent, [10, 9, 8]),
        },
        ShareBackground::Solid => {
            let bg = settings.solid_color;
            let fg = if luma(bg) > 0.55 {
                [24, 22, 20]
            } else {
                [246, 240, 228]
            };
            Palette {
                bg,
                bg_alt: shift(bg, if luma(bg) > 0.5 { -0.06 } else { 0.06 }),
                fg,
                dim: mix(fg, bg, 0.38),
                accent: ensure_visible(theme.accent, bg),
            }
        }
    }
}

fn paint_background(
    mode: ShareBackground,
    data: &ShareData,
    custom_bg: Option<&RgbaImage>,
    palette: &Palette,
    width: u32,
    height: u32,
) -> RgbaImage {
    let source = match mode {
        ShareBackground::Cover => data.cover.as_deref(),
        ShareBackground::Image => custom_bg,
        _ => None,
    };
    if let Some(source) = source {
        let (sigma, top_alpha) = if mode == ShareBackground::Cover {
            (0.06, 0.38)
        } else {
            (0.018, 0.28)
        };
        let mut image = blurred_fill(source, width, height, sigma);
        apply_scrim(&mut image, width, height, top_alpha);
        return image;
    }

    let mut image = gradient(width, height, palette.bg_alt, palette.bg);
    apply_vignette(&mut image, width, height);
    image
}

fn blurred_fill(source: &RgbaImage, width: u32, height: u32, sigma_ratio: f32) -> RgbaImage {
    let small_w = (width / 4).max(48);
    let small_h = (height / 4).max(48);
    let small = cover_crop(source, small_w, small_h);
    let blurred = imageops::blur(&small, (small_w.max(small_h) as f32 * sigma_ratio).max(0.8));
    imageops::resize(&blurred, width, height, imageops::FilterType::Lanczos3)
}

fn cover_crop(source: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    let (sw, sh) = source.dimensions();
    if sw == 0 || sh == 0 {
        return RgbaImage::new(width, height);
    }
    let scale = (width as f32 / sw as f32).max(height as f32 / sh as f32);
    let nw = (sw as f32 * scale).ceil().max(1.0) as u32;
    let nh = (sh as f32 * scale).ceil().max(1.0) as u32;
    let resized = imageops::resize(source, nw, nh, imageops::FilterType::Lanczos3);
    let x = (nw.saturating_sub(width)) / 2;
    let y = (nh.saturating_sub(height)) / 2;
    imageops::crop_imm(&resized, x, y, width, height).to_image()
}

fn gradient(width: u32, height: u32, top: [u8; 3], bottom: [u8; 3]) -> RgbaImage {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    let denominator = height.saturating_sub(1).max(1) as f32;
    for y in 0..height {
        let t = y as f32 / denominator;
        let color = mix(top, bottom, t);
        for _ in 0..width {
            data.extend_from_slice(&[color[0], color[1], color[2], 255]);
        }
    }
    RgbaImage::from_raw(width, height, data).unwrap_or_else(|| RgbaImage::new(width, height))
}

fn apply_vignette(image: &mut RgbaImage, width: u32, height: u32) {
    let cx = width as f32 * 0.5;
    let cy = height as f32 * 0.5;
    let max_dist = (cx * cx + cy * cy).sqrt().max(1.0);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        let dist = (dx * dx + dy * dy).sqrt() / max_dist;
        let factor = 1.0 - 0.10 * dist * dist;
        pixel[0] = (pixel[0] as f32 * factor).round() as u8;
        pixel[1] = (pixel[1] as f32 * factor).round() as u8;
        pixel[2] = (pixel[2] as f32 * factor).round() as u8;
    }
}

fn apply_scrim(image: &mut RgbaImage, width: u32, height: u32, top_alpha: f32) {
    let denominator = height.saturating_sub(1).max(1) as f32;
    let scrim = [8, 7, 6];
    let bottom_alpha = (top_alpha + 0.30).min(0.78);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let t = y as f32 / denominator;
        let alpha = top_alpha + (bottom_alpha - top_alpha) * t;
        let _ = x;
        let base = [pixel[0], pixel[1], pixel[2]];
        let out = mix(base, scrim, alpha);
        pixel[0] = out[0];
        pixel[1] = out[1];
        pixel[2] = out[2];
        pixel[3] = 255;
    }
    apply_vignette(image, width, height);
}

fn fill_rect(
    image: &mut RgbaImage,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: [u8; 3],
    alpha: f32,
) {
    for py in y.max(0)..(y + height.max(0)).min(image.height() as i32) {
        for px in x.max(0)..(x + width.max(0)).min(image.width() as i32) {
            blend_at(image, px, py, color, alpha);
        }
    }
}

fn blend_at(image: &mut RgbaImage, x: i32, y: i32, color: [u8; 3], alpha: f32) {
    if x < 0 || y < 0 || x >= image.width() as i32 || y >= image.height() as i32 {
        return;
    }
    let a = alpha.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    let pixel = image.get_pixel_mut(x as u32, y as u32);
    let base = [pixel[0], pixel[1], pixel[2]];
    let out = mix(base, color, a);
    pixel[0] = out[0];
    pixel[1] = out[1];
    pixel[2] = out[2];
    pixel[3] = 255;
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    let channel = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t).round() as u8;
    [
        channel(a[0], b[0]),
        channel(a[1], b[1]),
        channel(a[2], b[2]),
    ]
}

fn shift(color: [u8; 3], amount: f32) -> [u8; 3] {
    if amount >= 0.0 {
        mix(color, [255, 255, 255], amount)
    } else {
        mix(color, [0, 0, 0], -amount)
    }
}

fn luma(color: [u8; 3]) -> f32 {
    (0.2126 * color[0] as f32 + 0.7152 * color[1] as f32 + 0.0722 * color[2] as f32) / 255.0
}

fn ensure_visible(accent: [u8; 3], background: [u8; 3]) -> [u8; 3] {
    let difference = (luma(accent) - luma(background)).abs();
    if difference >= 0.22 {
        accent
    } else if luma(background) < 0.4 {
        mix(accent, [255, 245, 225], 0.45)
    } else {
        mix(accent, [20, 18, 16], 0.35)
    }
}

/// Carpeta por defecto para las imágenes exportadas.
pub fn default_dir() -> PathBuf {
    dirs::picture_dir()
        .or_else(dirs::download_dir)
        .unwrap_or_else(|| crate::store::data_dir().join("share"))
}

/// Guarda una imagen con un nombre legible y un temporal + rename.
pub fn save_png(image: &RgbaImage, dir: &Path, book: &str) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{}-{:03}", duration.as_secs(), duration.subsec_millis()))
        .unwrap_or_else(|_| "0-000".to_string());
    let name = format!("candil-cita-{}-{stamp}.png", slug(book));
    let path = dir.join(name);
    save_png_to(image, &path)?;
    Ok(path)
}

/// Guarda en una ruta exacta, creando el directorio si hace falta.
pub fn save_png_to(image: &RgbaImage, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("No pude crear {}: {e}", parent.display()))?;
    }
    let temp = path.with_file_name(format!(
        ".{}.{}.tmp.png",
        path.file_stem().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    if let Err(error) = image.save_with_format(&temp, ImageFormat::Png) {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("No pude escribir la imagen: {error}"));
    }
    if let Err(error) = std::fs::rename(&temp, path) {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("No pude guardar la imagen: {error}"));
    }
    Ok(())
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in text.trim().chars() {
        if ch.is_alphanumeric() {
            out.push(ch.to_lowercase().next().unwrap_or(ch));
            last_dash = false;
        } else if (ch.is_whitespace() || ch == '-' || ch == '_') && !last_dash {
            out.push('-');
            last_dash = true;
        }
        if out.chars().count() >= 48 {
            break;
        }
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "cita".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(quote: &str) -> ShareData {
        ShareData {
            quote: quote.into(),
            book: "La carretera".into(),
            author: "Cormac McCarthy".into(),
            chapter: Some("Capítulo 1".into()),
            cover: None,
        }
    }

    fn theme() -> ThemeColors {
        ThemeColors {
            bg: [242, 234, 216],
            fg: [43, 40, 35],
            dim: [125, 115, 98],
            accent: [165, 99, 50],
        }
    }

    #[test]
    fn todos_los_formatos_tienen_sus_dimensiones() {
        let expected = [
            (ShareFormat::Story, (270, 480)),
            (ShareFormat::Post, (270, 338)),
            (ShareFormat::Square, (270, 270)),
            (ShareFormat::Portrait, (270, 360)),
            (ShareFormat::Landscape, (480, 270)),
        ];
        for (format, dimensions) in expected {
            let settings = ShareSettings {
                format,
                show_chapter: true,
                ..ShareSettings::default()
            };
            let rendered =
                render(&data("Una cita breve."), &settings, &theme(), None, 0.25).unwrap();
            assert_eq!(rendered.image.dimensions(), dimensions, "{format:?}");
        }
    }

    #[test]
    fn una_cita_larga_se_recorta() {
        let long = "palabra ".repeat(400);
        let rendered = render(
            &data(&long),
            &ShareSettings::default(),
            &theme(),
            None,
            0.25,
        )
        .unwrap();
        assert!(rendered.truncated);
    }

    #[test]
    fn portada_sin_imagen_cae_a_papel() {
        let rendered = render(
            &data("Una cita."),
            &ShareSettings {
                background: ShareBackground::Cover,
                ..ShareSettings::default()
            },
            &theme(),
            None,
            0.25,
        )
        .unwrap();
        assert!(rendered.fallback);
    }

    #[test]
    fn nombre_de_archivo_es_seguro() {
        assert_eq!(slug("La carretera / Cormac"), "la-carretera-cormac");
        assert_eq!(slug("  ¿Qué?  "), "qué");
        assert_eq!(slug("///"), "cita");
        assert!(slug(&"a".repeat(100)).chars().count() <= 48);
    }

    #[test]
    fn guarda_png_en_una_ruta_temporal() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("candil-share-{}-{stamp}", std::process::id()));
        let image = RgbaImage::from_pixel(4, 4, image::Rgba([10, 20, 30, 255]));
        let path = save_png(&image, &dir, "Mi libro").unwrap();
        assert!(path.exists());
        assert!(path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("candil-cita-mi-libro-"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn color_solido_elige_texto_contrastante() {
        let light = palette_for(
            ShareBackground::Solid,
            &ShareSettings {
                solid_color: [245, 245, 245],
                ..ShareSettings::default()
            },
            &theme(),
        );
        assert!(luma(light.fg) < 0.2);
        let dark = palette_for(
            ShareBackground::Solid,
            &ShareSettings {
                solid_color: [20, 20, 20],
                ..ShareSettings::default()
            },
            &theme(),
        );
        assert!(luma(dark.fg) > 0.8);
    }

    #[test]
    fn imagen_propia_se_usa_como_fondo() {
        let custom = RgbaImage::from_pixel(64, 64, image::Rgba([120, 80, 40, 255]));
        let rendered = render(
            &data("Una cita."),
            &ShareSettings {
                background: ShareBackground::Image,
                ..ShareSettings::default()
            },
            &theme(),
            Some(&custom),
            0.25,
        )
        .unwrap();
        assert!(!rendered.fallback);
    }

    #[test]
    fn portada_disponible_se_desenfoca_y_se_usa() {
        let mut book = data("Una cita.");
        book.cover = Some(Arc::new(RgbaImage::from_pixel(
            32,
            48,
            image::Rgba([80, 40, 20, 255]),
        )));
        let rendered = render(
            &book,
            &ShareSettings {
                background: ShareBackground::Cover,
                ..ShareSettings::default()
            },
            &theme(),
            None,
            0.25,
        )
        .unwrap();
        assert!(!rendered.fallback);
    }

    #[test]
    fn cita_vacia_no_rompe_el_render() {
        let rendered = render(&data(""), &ShareSettings::default(), &theme(), None, 0.25).unwrap();
        assert_eq!(rendered.image.dimensions(), (270, 480));
        assert!(!rendered.truncated);
    }

    #[test]
    fn share_settings_hacen_roundtrip() {
        let settings = ShareSettings {
            format: ShareFormat::Post,
            background: ShareBackground::Solid,
            solid_color: [1, 2, 3],
            image_path: Some(PathBuf::from("/tmp/fondo.png")),
            show_book: false,
            show_author: true,
            show_chapter: true,
            show_mark: false,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let decoded: ShareSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, decoded);
        let old: ShareSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(old, ShareSettings::default());
    }
}
