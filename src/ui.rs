//! Componentes visuales compartidos. Viven acá para que biblioteca, lector y
//! paneles usen las mismas medidas, estados y colores en vez de repetir
//! `egui::Button`/`Frame` con valores literales.

use eframe::egui::{self, Color32, Response, RichText, Stroke, Ui, Vec2};

use crate::theme::{self, radius, space, text, Theme};

/// Anchos de panel: compacto, medio y configuración.
pub const PANEL_SM: f32 = 340.0;
pub const PANEL_MD: f32 = 420.0;
pub const PANEL_LG: f32 = 760.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Destructive,
}

fn min_height() -> Vec2 {
    Vec2::new(0.0, 32.0)
}

/// Botón con las cuatro variantes del sistema. `Primary` usa el acento del
/// tema; `Secondary` el panel y el borde; `Ghost` no tiene fondo en reposo;
/// `Destructive` usa el color de error.
pub fn button(ui: &mut Ui, th: &Theme, label: impl Into<String>, kind: ButtonKind) -> Response {
    let label = label.into();
    let body = RichText::new(label).size(text::BODY);

    match kind {
        ButtonKind::Primary => {
            ui.scope(|ui| {
                let visuals = ui.visuals_mut();
                visuals.widgets.inactive.weak_bg_fill = th.accent;
                visuals.widgets.hovered.weak_bg_fill = th.accent.gamma_multiply(0.92);
                visuals.widgets.active.weak_bg_fill = th.accent.gamma_multiply(0.84);
                visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                visuals.widgets.active.bg_stroke = Stroke::new(2.0, th.fg);
                ui.add(
                    egui::Button::new(body.color(th.bg).strong())
                        .min_size(min_height())
                        .corner_radius(radius::MD),
                )
            })
            .inner
        }
        ButtonKind::Secondary => ui.add(
            egui::Button::new(body.color(th.fg))
                .min_size(min_height())
                .stroke(Stroke::new(1.0, th.border))
                .corner_radius(radius::MD),
        ),
        ButtonKind::Ghost => {
            ui.scope(|ui| {
                let visuals = ui.visuals_mut();
                visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                visuals.widgets.hovered.weak_bg_fill = th.accent.gamma_multiply(0.12);
                visuals.widgets.active.weak_bg_fill = th.accent.gamma_multiply(0.20);
                visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                visuals.widgets.active.bg_stroke = Stroke::new(2.0, th.accent);
                ui.add(
                    egui::Button::new(body.color(th.fg))
                        .min_size(min_height())
                        .corner_radius(radius::MD),
                )
            })
            .inner
        }
        ButtonKind::Destructive => {
            ui.scope(|ui| {
                let visuals = ui.visuals_mut();
                visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                visuals.widgets.hovered.weak_bg_fill = th.error.gamma_multiply(0.14);
                visuals.widgets.active.weak_bg_fill = th.error.gamma_multiply(0.24);
                visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, th.error);
                visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, th.error);
                visuals.widgets.active.bg_stroke = Stroke::new(2.0, th.error);
                ui.add(
                    egui::Button::new(body.color(th.error))
                        .min_size(min_height())
                        .corner_radius(radius::MD),
                )
            })
            .inner
        }
    }
}

/// Botón de icono con tooltip. Mantiene el objetivo mínimo y el foco de teclado
/// aunque el contenido sea un solo glifo.
pub fn icon_button(ui: &mut Ui, th: &Theme, icon: &str, tooltip: &str) -> Response {
    let response = ui.add(
        egui::Button::new(RichText::new(icon).size(text::BODY).color(th.fg))
            .min_size(Vec2::splat(32.0))
            .corner_radius(radius::MD),
    );
    response.on_hover_text(tooltip)
}

/// Frame común de panel: superficie, borde hairline y padding de 16.
pub fn panel_frame(th: &Theme) -> egui::Frame {
    panel_frame_fill(th, th.panel)
}

/// Igual que `panel_frame`, pero con otra superficie. La usa Consulta para
/// mantener su tono elevado sin romper geometría ni bordes.
pub fn panel_frame_fill(th: &Theme, fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, th.border))
        .corner_radius(radius::LG)
        .inner_margin(space::LG)
}

/// Título de sección dentro de un panel o de configuración.
pub fn section_header(ui: &mut Ui, th: &Theme, title: &str) {
    ui.add_space(space::SM);
    ui.label(RichText::new(title).size(text::TITLE).strong().color(th.fg));
    ui.add_space(space::XS);
}

/// Campo de texto de una línea con ancho flexible.
pub fn input(ui: &mut Ui, text: &mut String, hint: &str) -> Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .desired_width(f32::INFINITY),
    )
}

/// Chip de carpeta o de filtro. El estado seleccionado usa borde y peso, no
/// solo color.
pub fn chip(ui: &mut Ui, th: &Theme, label: &str, selected: bool) -> Response {
    let fill = if selected {
        th.accent.gamma_multiply(0.16)
    } else {
        th.panel
    };
    let stroke = if selected {
        Stroke::new(1.0, th.accent)
    } else {
        Stroke::new(1.0, th.border)
    };
    let response = ui.add(
        egui::Button::new(RichText::new(label).size(text::META).color(if selected {
            th.fg
        } else {
            th.dim
        }))
        .fill(fill)
        .stroke(stroke)
        .min_size(Vec2::new(0.0, 32.0))
        .corner_radius(radius::SM),
    );
    response
}

/// Preview de tema: rectángulo con `Aa` en los colores del preset. Se pinta
/// sobre un botón para conservar foco y activación por teclado.
pub fn theme_swatch(ui: &mut Ui, th: &Theme, target: &Theme, selected: bool) -> Response {
    let response = ui.add(
        egui::Button::new("")
            .frame(false)
            .min_size(Vec2::new(50.0, 32.0)),
    );
    let rect = response.rect;
    ui.painter().rect_filled(rect, radius::SM, target.bg);
    let stroke = if selected || response.has_focus() {
        Stroke::new(2.0, th.accent)
    } else {
        Stroke::new(1.0, th.dim)
    };
    ui.painter()
        .rect_stroke(rect, radius::SM, stroke, egui::StrokeKind::Inside);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Aa",
        egui::FontId::new(15.0, egui::FontFamily::Proportional),
        target.fg,
    );
    response
}

/// Swatch de color pintado: círculo relleno + anillo. Evita el glifo `●`, que
/// Literata no incluye y caía a un cuadrado hueco.
pub fn color_swatch(
    ui: &mut Ui,
    th: &Theme,
    color: Color32,
    selected: bool,
    tooltip: &str,
) -> Response {
    let response = ui.add(
        egui::Button::new("")
            .frame(false)
            .min_size(Vec2::splat(32.0)),
    );
    let center = response.rect.center();
    let radius = 7.0;

    ui.painter().circle_filled(center, radius, color);
    if selected {
        ui.painter()
            .circle_stroke(center, radius + 2.0, Stroke::new(2.0, th.fg));
    } else if response.has_focus() {
        ui.painter()
            .circle_stroke(center, radius + 2.0, Stroke::new(2.0, th.accent));
    } else if response.hovered() {
        ui.painter()
            .circle_stroke(center, radius + 2.0, Stroke::new(1.0, th.accent));
    } else {
        ui.painter()
            .circle_stroke(center, radius + 2.0, Stroke::new(1.0, th.border));
    }

    response.on_hover_text(tooltip)
}

/// Atajo para textos de metadata que no son acciones.
pub fn meta(ui: &mut Ui, th: &Theme, text_value: impl Into<String>) -> Response {
    ui.label(
        RichText::new(text_value.into())
            .size(text::META)
            .color(th.dim),
    )
}

/// Atajo para labels de formulario.
pub fn label(ui: &mut Ui, th: &Theme, text_value: impl Into<String>) -> Response {
    ui.label(
        RichText::new(text_value.into())
            .size(text::LABEL)
            .color(th.dim),
    )
}

/// Separa visualmente secciones sin agregar decoración.
pub fn section_gap(ui: &mut Ui) {
    ui.add_space(theme::space::MD);
}
