//! Tipografía embebida: Literata (OFL) en cuatro pesos + familia monoespaciada.

use std::sync::Arc;

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

pub const SERIF: &str = "lit";
pub const SERIF_BOLD: &str = "lit-bold";
pub const SERIF_ITALIC: &str = "lit-italic";
pub const SERIF_BOLD_ITALIC: &str = "lit-bold-italic";

fn add_family(defs: &mut FontDefinitions, key: &str, faces: &[&str]) {
    defs.families.insert(
        FontFamily::Name(key.into()),
        faces.iter().map(|s| s.to_string()).collect(),
    );
}

pub fn install(ctx: &Context) {
    let mut defs = FontDefinitions::default();

    let faces: [(&str, &[u8]); 5] = [
        (
            "lit-regular",
            include_bytes!("../assets/fonts/Literata-Regular.ttf"),
        ),
        (
            "lit-semibold",
            include_bytes!("../assets/fonts/Literata-SemiBold.ttf"),
        ),
        (
            "lit-boldf",
            include_bytes!("../assets/fonts/Literata-Bold.ttf"),
        ),
        (
            "lit-italicf",
            include_bytes!("../assets/fonts/Literata-Italic.ttf"),
        ),
        (
            "lit-bolditalicf",
            include_bytes!("../assets/fonts/Literata-BoldItalic.ttf"),
        ),
    ];
    for (name, bytes) in faces {
        defs.font_data
            .insert(name.to_string(), Arc::new(FontData::from_static(bytes)));
    }

    add_family(&mut defs, SERIF, &["lit-regular", "lit-semibold"]);
    add_family(&mut defs, SERIF_BOLD, &["lit-boldf"]);
    add_family(&mut defs, SERIF_ITALIC, &["lit-italicf"]);
    add_family(&mut defs, SERIF_BOLD_ITALIC, &["lit-bolditalicf"]);

    // La UI también respira Literata; si falta un glifo, cae a la fuente por defecto.
    if let Some(list) = defs.families.get_mut(&FontFamily::Proportional) {
        list.insert(0, "lit-regular".to_string());
        list.push("lit-semibold".to_string());
    }

    ctx.set_fonts(defs);
}

pub fn family(bold: bool, italic: bool) -> FontFamily {
    let key = match (bold, italic) {
        (true, true) => SERIF_BOLD_ITALIC,
        (true, false) => SERIF_BOLD,
        (false, true) => SERIF_ITALIC,
        (false, false) => SERIF,
    };
    FontFamily::Name(key.into())
}
