use eframe::egui::{self, Color32, Stroke};

/// Escala de espaciado de UI. El ritmo de lectura no usa esta escala: se
/// deriva del tamaño de fuente y del interlineado.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
    pub const XXXL: f32 = 48.0;
    pub const HUGE: f32 = 64.0;
}

/// Radios de la interfaz: chips/inputs, botones/tarjetas y paneles.
pub mod radius {
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
}

/// Tamaños de la UI. La lectura se ajusta aparte desde `Settings`.
pub mod text {
    pub const DISPLAY: f32 = 24.0;
    pub const HEADLINE: f32 = 18.0;
    pub const TITLE: f32 = 16.0;
    pub const BODY: f32 = 13.5;
    pub const BODY_SMALL: f32 = 12.5;
    pub const LABEL: f32 = 12.0;
    pub const META: f32 = 11.0;
    pub const ICON: f32 = 14.0;
}

/// Un preset de lectura: solo colores. Tipografía y medidas viven en `Settings`.
#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color32,
    pub fg: Color32,
    /// Texto secundario y metadata. Debe cumplir AA sobre `bg` y `panel`.
    pub dim: Color32,
    pub accent: Color32,
    pub panel: Color32,
    /// Errores y acciones destructivas. Debe cumplir AA sobre `bg` y `panel`.
    pub error: Color32,
    /// Fondo resuelto de la selección de texto (acento al 35% sobre `bg`).
    pub selection: Color32,
    /// Borde hairline resuelto (dim al 35% sobre `bg`).
    pub border: Color32,
}

const fn c(hex: u32) -> Color32 {
    Color32::from_rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

const fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (a.r() as f32 * t + b.r() as f32 * (1.0 - t)) as u8,
        (a.g() as f32 * t + b.g() as f32 * (1.0 - t)) as u8,
        (a.b() as f32 * t + b.b() as f32 * (1.0 - t)) as u8,
    )
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Papel",
        bg: c(0xF4ECD8),
        fg: c(0x3B3226),
        dim: c(0x6E624A),
        accent: c(0x8A582A),
        panel: c(0xEADFC6),
        error: c(0x9C4B43),
        selection: c(0xCFB89B),
        border: c(0xC5BCA6),
    },
    Theme {
        name: "Noche",
        bg: c(0x14161A),
        fg: c(0xC9C7C1),
        dim: c(0x84878E),
        accent: c(0x7FA7C8),
        panel: c(0x1C1F25),
        error: c(0xC56E65),
        selection: c(0x394957),
        border: c(0x3B3E43),
    },
    Theme {
        name: "Claro",
        bg: c(0xFBFAF7),
        fg: c(0x24221F),
        dim: c(0x6F6C66),
        accent: c(0x3F6E9A),
        panel: c(0xF0EEE8),
        error: c(0xA95249),
        selection: c(0xB9C9D6),
        border: c(0xCAC8C4),
    },
    Theme {
        name: "Ámbar",
        bg: c(0x1E1710),
        fg: c(0xDACBA9),
        dim: c(0x93856B),
        accent: c(0xD9A441),
        panel: c(0x271E15),
        error: c(0xC56E65),
        selection: c(0x5F4821),
        border: c(0x473E30),
    },
    Theme {
        name: "Contraste",
        bg: c(0x000000),
        fg: c(0xFFFFFF),
        dim: c(0x9A9A9A),
        accent: c(0xE8C547),
        panel: c(0x101010),
        error: c(0xBF5F55),
        selection: c(0x514519),
        border: c(0x363636),
    },
];

pub fn theme(index: usize) -> Theme {
    THEMES[index.min(THEMES.len() - 1)]
}

fn widget(
    bg_fill: Color32,
    weak_bg_fill: Color32,
    fg: Color32,
    bg_stroke: Stroke,
) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill,
        weak_bg_fill,
        bg_stroke,
        corner_radius: radius::MD.into(),
        fg_stroke: Stroke::new(1.0, fg),
        expansion: 0.0,
    }
}

impl Theme {
    pub fn is_dark(&self) -> bool {
        let l =
            0.299 * self.bg.r() as f32 + 0.587 * self.bg.g() as f32 + 0.114 * self.bg.b() as f32;
        l < 128.0
    }

    pub fn apply(&self, ctx: &eframe::egui::Context) {
        let mut visuals = if self.is_dark() {
            eframe::egui::Visuals::dark()
        } else {
            eframe::egui::Visuals::light()
        };

        // Superficies y texto.
        visuals.panel_fill = self.panel;
        visuals.window_fill = self.panel;
        visuals.extreme_bg_color = self.bg;
        visuals.text_edit_bg_color = Some(self.bg);
        visuals.faint_bg_color = self.panel;
        visuals.code_bg_color = self.panel;
        visuals.override_text_color = None;
        visuals.weak_text_color = Some(self.dim);
        visuals.hyperlink_color = self.accent;
        visuals.warn_fg_color = self.error;
        visuals.error_fg_color = self.error;

        // Selección y foco. El foco usa el estado `active` de egui: 2 px de
        // acento, visible en los cinco temas.
        visuals.selection.bg_fill = self.selection;
        visuals.selection.stroke = Stroke::new(1.0, self.fg);

        // Superficies planas: tono + borde hairline, sin sombras decorativas.
        visuals.window_corner_radius = radius::LG.into();
        visuals.menu_corner_radius = radius::MD.into();
        visuals.window_stroke = Stroke::new(1.0, self.border);
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.popup_shadow = egui::epaint::Shadow::NONE;

        // Widgets: los sliders, checkboxes y botones dejan de usar el azul de
        // egui. `active` también pinta el anillo de foco.
        let track = mix(self.dim, self.bg, 0.25);
        let hover_track = mix(self.accent, self.bg, 0.30);
        let active_track = mix(self.accent, self.bg, 0.45);
        let hover_fill = mix(self.accent, self.panel, 0.12);
        let active_fill = mix(self.accent, self.panel, 0.22);
        let hover_border = mix(self.accent, self.border, 0.60);

        visuals.widgets.noninteractive = widget(
            self.panel,
            self.panel,
            self.fg,
            Stroke::new(1.0, self.border),
        );
        visuals.widgets.inactive =
            widget(track, self.panel, self.fg, Stroke::new(1.0, self.border));
        visuals.widgets.hovered = widget(
            hover_track,
            hover_fill,
            self.fg,
            Stroke::new(1.0, hover_border),
        );
        visuals.widgets.active = widget(
            active_track,
            active_fill,
            self.fg,
            Stroke::new(2.0, self.accent),
        );
        visuals.widgets.open = widget(
            hover_track,
            hover_fill,
            self.fg,
            Stroke::new(1.0, hover_border),
        );

        let mut style = (*ctx.style()).clone();
        style.visuals = visuals;
        style.spacing.item_spacing = eframe::egui::vec2(space::SM, 6.0);
        style.spacing.button_padding = eframe::egui::vec2(10.0, 6.0);
        // Sin animaciones de apertura/hover: la interfaz cambia de estado sin
        // movimiento y las capturas son deterministas.
        style.animation_time = 0.0;
        ctx.set_style(style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn channel(value: u8) -> f32 {
        let value = value as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    fn luminance(color: Color32) -> f32 {
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn todos_los_temas_cumplen_aa_en_texto() {
        for theme in THEMES {
            for (label, fg, bg) in [
                ("fg/bg", theme.fg, theme.bg),
                ("dim/bg", theme.dim, theme.bg),
                ("dim/panel", theme.dim, theme.panel),
                ("accent/bg", theme.accent, theme.bg),
                ("accent/panel", theme.accent, theme.panel),
                ("error/bg", theme.error, theme.bg),
                ("error/panel", theme.error, theme.panel),
                ("fg/selection", theme.fg, theme.selection),
            ] {
                let ratio = contrast(fg, bg);
                assert!(
                    ratio >= 4.5,
                    "{}: {label} = {ratio:.2}:1 (mínimo 4.5:1)",
                    theme.name
                );
            }
        }
    }

    #[test]
    fn mix_es_constante_y_acotado() {
        let a = c(0x000000);
        let b = c(0xFFFFFF);
        assert_eq!(mix(a, b, 0.0), b);
        assert_eq!(mix(a, b, 1.0), a);
    }
}
