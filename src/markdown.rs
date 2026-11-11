//! Markdown mínimo para respuestas de IA y notas.
//!
//! No es un parser completo: cubre lo que producen los modelos y lo que usa el
//! usuario en una nota: párrafos, encabezados, listas, citas, bloques de código,
//! separadores y énfasis inline (`**negrita**`, `*itálica*`, `` `código` ``).
//! Se renderiza con el sistema de la app, sin motor CSS ni dependencias.

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{self, Color32, FontFamily, FontId, Label, RichText, Stroke, Ui};

use crate::fonts;
use crate::theme::{self, Theme};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Block {
    Paragraph(String),
    Heading(usize, String),
    Bullet(String),
    Numbered(String, String),
    Quote(String),
    Code(String),
    Rule,
}

pub fn show(ui: &mut Ui, th: &Theme, text: &str, size: f32, color: Color32) {
    for block in parse(text) {
        match block {
            Block::Paragraph(text) => {
                ui.add(Label::new(inline_job(&text, size, color, false, false, th)).wrap());
                ui.add_space(theme::space::XS);
            }
            Block::Heading(level, text) => {
                let heading_size = match level {
                    1 => size + 5.0,
                    2 => size + 3.5,
                    3 => size + 2.0,
                    _ => size + 1.0,
                };
                ui.add(Label::new(inline_job(&text, heading_size, color, true, false, th)).wrap());
                ui.add_space(theme::space::XS);
            }
            Block::Bullet(text) => {
                let mut job = LayoutJob::default();
                job.append(
                    "•  ",
                    0.0,
                    TextFormat {
                        font_id: FontId::new(size, fonts::family(false, false)),
                        color: th.accent,
                        ..Default::default()
                    },
                );
                append_inline(&mut job, &text, size, color, false, false, th);
                ui.add(Label::new(job).wrap());
                ui.add_space(2.0);
            }
            Block::Numbered(number, text) => {
                let mut job = LayoutJob::default();
                job.append(
                    &format!("{number}. "),
                    0.0,
                    TextFormat {
                        font_id: FontId::new(size, fonts::family(false, false)),
                        color: th.accent,
                        ..Default::default()
                    },
                );
                append_inline(&mut job, &text, size, color, false, false, th);
                ui.add(Label::new(job).wrap());
                ui.add_space(2.0);
            }
            Block::Quote(text) => {
                egui::Frame::new()
                    .fill(th.bg)
                    .stroke(Stroke::new(1.0, th.border))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .corner_radius(theme::radius::SM)
                    .show(ui, |ui| {
                        ui.add(Label::new(inline_job(&text, size, th.dim, false, true, th)).wrap());
                    });
                ui.add_space(theme::space::XS);
            }
            Block::Code(code) => {
                egui::Frame::new()
                    .fill(th.bg)
                    .stroke(Stroke::new(1.0, th.border))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .corner_radius(theme::radius::SM)
                    .show(ui, |ui| {
                        ui.add(
                            Label::new(
                                RichText::new(code)
                                    .monospace()
                                    .size((size - 1.0).max(9.0))
                                    .color(th.fg),
                            )
                            .wrap(),
                        );
                    });
                ui.add_space(theme::space::XS);
            }
            Block::Rule => {
                ui.add_space(theme::space::XS);
                ui.separator();
                ui.add_space(theme::space::XS);
            }
        }
    }
}

/// Renderiza una sola línea con énfasis inline; para vistas previas compactas.
pub fn inline(ui: &mut Ui, th: &Theme, text: &str, size: f32, color: Color32) {
    ui.add(Label::new(inline_job(text, size, color, false, false, th)).wrap());
}

fn parse(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut code = Vec::new();
    let mut in_code = false;

    for raw in text.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            if in_code {
                blocks.push(Block::Code(code.join("\n")));
                code.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }
        if in_code {
            code.push(raw.to_string());
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if matches!(trimmed, "---" | "***" | "___") {
            blocks.push(Block::Rule);
            continue;
        }
        if let Some((level, rest)) = heading(trimmed) {
            blocks.push(Block::Heading(level, rest.to_string()));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix('>') {
            blocks.push(Block::Quote(rest.trim().to_string()));
            continue;
        }
        if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        {
            blocks.push(Block::Bullet(rest.trim().to_string()));
            continue;
        }
        if let Some((number, rest)) = numbered(trimmed) {
            blocks.push(Block::Numbered(number.to_string(), rest.trim().to_string()));
            continue;
        }
        blocks.push(Block::Paragraph(line.to_string()));
    }

    if in_code && !code.is_empty() {
        blocks.push(Block::Code(code.join("\n")));
    }
    blocks
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = line[level..].strip_prefix(' ')?;
    Some((level, rest.trim()))
}

fn numbered(line: &str) -> Option<(&str, &str)> {
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let rest = &line[digits..];
    let rest = rest.strip_prefix('.').or_else(|| rest.strip_prefix(')'))?;
    let rest = rest.strip_prefix(' ')?;
    Some((&line[..digits], rest))
}

fn inline_job(
    text: &str,
    size: f32,
    color: Color32,
    bold: bool,
    italic: bool,
    th: &Theme,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    append_inline(&mut job, text, size, color, bold, italic, th);
    job
}

fn append_inline(
    job: &mut LayoutJob,
    text: &str,
    size: f32,
    color: Color32,
    mut bold: bool,
    mut italic: bool,
    th: &Theme,
) {
    let mut rest = text;
    while !rest.is_empty() {
        let code_at = rest.find('`');
        let bold_at = rest.find("**");
        let italic_at = rest.find('*');

        let mut next: Option<(usize, usize, u8)> = None;
        if let Some(at) = code_at {
            next = Some((at, 1, 0));
        }
        if let Some(at) = bold_at {
            if next.is_none_or(|(current, _, _)| at < current) {
                next = Some((at, 2, 1));
            }
        }
        if let Some(at) = italic_at {
            if next.is_none_or(|(current, _, _)| at < current) {
                next = Some((at, 1, 2));
            }
        }

        let Some((at, len, kind)) = next else {
            push_span(job, rest, size, color, bold, italic, th, false);
            break;
        };
        push_span(job, &rest[..at], size, color, bold, italic, th, false);
        match kind {
            0 => {
                let after = &rest[at + len..];
                if let Some(end) = after.find('`') {
                    push_span(job, &after[..end], size, th.accent, false, false, th, true);
                    rest = &after[end + 1..];
                } else {
                    push_span(job, "`", size, color, bold, italic, th, false);
                    rest = after;
                }
            }
            1 => {
                bold = !bold;
                rest = &rest[at + len..];
            }
            _ => {
                italic = !italic;
                rest = &rest[at + len..];
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn push_span(
    job: &mut LayoutJob,
    text: &str,
    size: f32,
    color: Color32,
    bold: bool,
    italic: bool,
    th: &Theme,
    code: bool,
) {
    if text.is_empty() {
        return;
    }
    let font_id = if code {
        FontId::new((size - 1.0).max(9.0), FontFamily::Monospace)
    } else {
        FontId::new(size, fonts::family(bold, italic))
    };
    job.append(
        text,
        0.0,
        TextFormat {
            font_id,
            color: if code { th.accent } else { color },
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_bloques_basicos() {
        let blocks = parse(
            "# Título\n\nUn párrafo con **negrita**.\n- uno\n- dos\n1. primero\n> cita\n---\n```\ncode\n```",
        );
        assert!(matches!(&blocks[0], Block::Heading(1, t) if t == "Título"));
        assert!(matches!(&blocks[1], Block::Paragraph(t) if t.contains("negrita")));
        assert!(matches!(&blocks[2], Block::Bullet(t) if t == "uno"));
        assert!(matches!(&blocks[3], Block::Bullet(t) if t == "dos"));
        assert!(matches!(&blocks[4], Block::Numbered(n, t) if n == "1" && t == "primero"));
        assert!(matches!(&blocks[5], Block::Quote(t) if t == "cita"));
        assert_eq!(blocks[6], Block::Rule);
        assert!(matches!(&blocks[7], Block::Code(t) if t == "code"));
    }

    #[test]
    fn inline_quita_marcas_y_aplica_estilos() {
        let th = theme::theme(0);
        let job = inline_job("hola **mundo** y *adiós*", 12.0, th.fg, false, false, &th);
        assert_eq!(job.text, "hola mundo y adiós");
        assert!(job.sections.len() >= 4);
        assert_eq!(
            job.sections[0].format.font_id.family,
            fonts::family(false, false)
        );
        assert_eq!(
            job.sections[1].format.font_id.family,
            fonts::family(true, false)
        );
        assert_eq!(
            job.sections[2].format.font_id.family,
            fonts::family(false, false)
        );
        assert_eq!(
            job.sections[3].format.font_id.family,
            fonts::family(false, true)
        );
    }

    #[test]
    fn inline_code_usa_monoespaciada() {
        let th = theme::theme(0);
        let job = inline_job("usá `cargo test` ahora", 12.0, th.fg, false, false, &th);
        assert_eq!(job.text, "usá cargo test ahora");
        assert!(job
            .sections
            .iter()
            .any(|section| section.format.font_id.family == FontFamily::Monospace));
    }

    #[test]
    fn marcas_sin_cerrar_quedan_como_texto() {
        let th = theme::theme(0);
        let job = inline_job("un *solo marcador", 12.0, th.fg, false, false, &th);
        assert_eq!(job.text, "un solo marcador");
    }
}
