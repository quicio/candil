use eframe::egui::Color32;

/// Un preset de lectura: solo colores. Tipografía y medidas viven en `Settings`.
#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color32,
    pub fg: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub panel: Color32,
}

const fn c(hex: u32) -> Color32 {
    Color32::from_rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

pub const THEMES: &[Theme] = &[
    Theme {
        name: "Papel",
        bg: c(0xF4ECD8),
        fg: c(0x3B3226),
        dim: c(0x857659),
        accent: c(0x96602E),
        panel: c(0xEADFC6),
    },
    Theme {
        name: "Noche",
        bg: c(0x14161A),
        fg: c(0xC9C7C1),
        dim: c(0x6C7079),
        accent: c(0x7FA7C8),
        panel: c(0x1C1F25),
    },
    Theme {
        name: "Claro",
        bg: c(0xFBFAF7),
        fg: c(0x24221F),
        dim: c(0x8B877F),
        accent: c(0x3F6E9A),
        panel: c(0xF0EEE8),
    },
    Theme {
        name: "Ámbar",
        bg: c(0x1E1710),
        fg: c(0xDACBA9),
        dim: c(0x8A7A5E),
        accent: c(0xD9A441),
        panel: c(0x271E15),
    },
    Theme {
        name: "Contraste",
        bg: c(0x000000),
        fg: c(0xFFFFFF),
        dim: c(0x9A9A9A),
        accent: c(0xE8C547),
        panel: c(0x101010),
    },
];

pub fn theme(index: usize) -> Theme {
    THEMES[index.min(THEMES.len() - 1)]
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
        visuals.panel_fill = self.panel;
        visuals.window_fill = self.panel;
        visuals.extreme_bg_color = self.bg;
        visuals.override_text_color = Some(self.fg);
        visuals.selection.bg_fill = self.accent.gamma_multiply(0.45);
        visuals.selection.stroke = eframe::egui::Stroke::new(1.0, self.fg);
        visuals.widgets.noninteractive.bg_stroke =
            eframe::egui::Stroke::new(1.0, self.dim.gamma_multiply(0.35));
        visuals.widgets.inactive.weak_bg_fill = self.panel;
        visuals.widgets.hovered.weak_bg_fill = self.accent.gamma_multiply(0.25);
        visuals.widgets.active.weak_bg_fill = self.accent.gamma_multiply(0.4);
        let mut style = (*ctx.style()).clone();
        style.visuals = visuals;
        style.spacing.item_spacing = eframe::egui::vec2(8.0, 6.0);
        style.spacing.button_padding = eframe::egui::vec2(8.0, 4.0);
        ctx.set_style(style);
    }
}
