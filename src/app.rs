//! Ventana y lectura: barra mínima, columna centrada, scroll continuo.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Key, Layout, RichText, ScrollArea, Sense, Stroke, Ui,
    Vec2,
};

use crate::config::{Config, Progress, Settings};
use crate::epub::Book;
use crate::fonts;
use crate::theme::{self, Theme, THEMES};
use crate::xhtml::{Block, Kind};

pub struct Lector {
    book: Option<Book>,
    chapter: usize,
    blocks: Vec<Block>,
    textures: HashMap<String, egui::TextureHandle>,
    cfg: Config,
    scroll: f32,
    max_scroll: f32,
    delta: f32,
    viewport_h: f32,
    show_toc: bool,
    show_settings: bool,
    status: Option<(String, Instant)>,
    applied_theme: Option<usize>,
    dirty: bool,
    last_save: Instant,
    dev: DevOpts,
    shot_frames: u32,
}

/// Banderas de desarrollo: captura e inspección visual sin tocar la UI.
#[derive(Debug, Clone, Default)]
pub struct DevOpts {
    pub shot: Option<PathBuf>,
    pub chapter: Option<usize>,
    pub panel: Option<String>,
}

impl Lector {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>, dev: DevOpts) -> Self {
        fonts::install(&cc.egui_ctx);
        let cfg = Config::load();
        let th = theme::theme(cfg.settings.theme);
        th.apply(&cc.egui_ctx);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(14.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            FontId::new(14.5, FontFamily::Proportional),
        );
        cc.egui_ctx.set_style(style);

        let theme_idx = cfg.settings.theme;
        let mut app = Self {
            book: None,
            chapter: 0,
            blocks: Vec::new(),
            textures: HashMap::new(),
            cfg,
            scroll: 0.0,
            max_scroll: 0.0,
            delta: 0.0,
            viewport_h: 600.0,
            show_toc: false,
            show_settings: false,
            status: None,
            applied_theme: Some(theme_idx),
            dirty: false,
            last_save: Instant::now(),
            dev,
            shot_frames: 0,
        };
        app.applied_theme = Some(app.cfg.settings.theme);

        let target = path.or_else(|| app.cfg.last_book.clone().map(PathBuf::from));
        if let Some(p) = target {
            if p.exists() {
                app.open(p);
            }
        }
        app
    }

    // -- libro ---------------------------------------------------------------

    fn open(&mut self, path: PathBuf) {
        match Book::open(&path) {
            Ok(book) => {
                let key = book.path.to_string_lossy().to_string();
                let prog = self.cfg.progress.get(&key).copied().unwrap_or_default();
                let n = book.spine.len();
                let title = book.title.clone();
                let author = book.author.clone();
                self.chapter = prog.chapter.min(n.saturating_sub(1));
                self.book = Some(book);
                self.cfg.last_book = Some(key.clone());
                self.cfg.touch_recent(&key);
                if let Some(ch) = self.dev.chapter {
                    self.chapter = ch.min(n.saturating_sub(1));
                }
                self.load_chapter(self.chapter, prog.offset);
                match self.dev.panel.as_deref() {
                    Some("toc") => self.show_toc = true,
                    Some("settings") => self.show_settings = true,
                    _ => {}
                }
                let _ = (title, author);
                self.dirty = true;
            }
            Err(e) => {
                self.status = Some((format!("No pude abrir el archivo: {e}"), Instant::now()));
            }
        }
    }

    fn load_chapter(&mut self, index: usize, offset: f32) {
        let blocks = match self.book.as_mut() {
            Some(b) => b.chapter_blocks(index),
            None => Vec::new(),
        };
        self.blocks = blocks;
        self.chapter = index;
        self.textures.clear();
        self.scroll = offset;
        self.delta = 0.0;
        self.dirty = true;
    }

    fn goto_chapter(&mut self, index: i64) {
        let n = match &self.book {
            Some(b) => b.chapter_count(),
            None => return,
        };
        let cur = self.chapter as i64;
        let next = (cur + index).clamp(0, n as i64 - 1) as usize;
        if next != self.chapter {
            self.load_chapter(next, 0.0);
        }
    }

    fn progress(&self) -> f32 {
        let n = self
            .book
            .as_ref()
            .map(|b| b.chapter_count())
            .unwrap_or(0)
            .max(1);
        let within = if self.max_scroll > 1.0 {
            (self.scroll / self.max_scroll).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (((self.chapter as f32) + within) / n as f32).clamp(0.0, 1.0)
    }

    fn persist(&mut self) {
        if let (Some(book), true) = (&self.book, self.dirty) {
            let key = book.path.to_string_lossy().to_string();
            self.cfg.progress.insert(
                key,
                Progress {
                    chapter: self.chapter,
                    offset: self.scroll,
                },
            );
        }
        self.cfg.settings.clamp();
        self.cfg.save();
        self.dirty = false;
        self.last_save = Instant::now();
    }

    // -- entrada -------------------------------------------------------------

    fn handle_input(&mut self, ctx: &egui::Context) {
        let dropped: Option<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .find(|p| {
                    p.extension()
                        .map(|e| e.eq_ignore_ascii_case("epub"))
                        .unwrap_or(false)
                })
        });
        if let Some(p) = dropped {
            self.open(p);
        }

        let typing = ctx.wants_keyboard_input();

        let (
            space,
            pgdn,
            pgup,
            up,
            down,
            left,
            right,
            plus,
            minus,
            key_t,
            key_s,
            key_esc,
            key_o,
            ctrl,
            home,
        ) = ctx.input(|i| {
            (
                i.key_pressed(Key::Space),
                i.key_pressed(Key::PageDown),
                i.key_pressed(Key::PageUp),
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
                i.key_pressed(Key::ArrowLeft),
                i.key_pressed(Key::ArrowRight),
                i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals),
                i.key_pressed(Key::Minus),
                i.key_pressed(Key::T),
                i.key_pressed(Key::S),
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::O),
                i.modifiers.command || i.modifiers.ctrl,
                i.key_pressed(Key::Home),
            )
        });

        if ctrl && key_o {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("EPUB", &["epub"])
                .pick_file()
            {
                self.open(p);
            }
        }

        if key_esc {
            self.show_toc = false;
            self.show_settings = false;
        }

        if self.book.is_none() || typing {
            return;
        }

        let page = self.viewport_h.max(200.0);
        if space || pgdn {
            self.delta += page * 0.86;
        }
        if pgup {
            self.delta -= page * 0.86;
        }
        if down {
            self.delta += 70.0;
        }
        if up {
            self.delta -= 70.0;
        }
        if home {
            self.scroll = 0.0;
            self.delta = 0.0;
        }
        if right {
            self.goto_chapter(1);
        }
        if left {
            self.goto_chapter(-1);
        }
        if plus {
            self.cfg.settings.font_size += 1.0;
            self.dirty = true;
        }
        if minus {
            self.cfg.settings.font_size -= 1.0;
            self.dirty = true;
        }
        if key_t {
            self.show_toc = !self.show_toc;
            self.show_settings = false;
        }
        if key_s {
            self.show_settings = !self.show_settings;
            self.show_toc = false;
        }
    }

    // -- pantallas -----------------------------------------------------------

    fn open_screen(&mut self, ctx: &egui::Context, th: &Theme) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(th.bg))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(90.0);
                    ui.label(RichText::new("lector").size(46.0).color(th.fg));
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("EPUB, sin adornos")
                            .size(15.0)
                            .color(th.dim)
                            .italics(),
                    );
                    ui.add_space(46.0);
                    if ui
                        .button(RichText::new("  Abrir un EPUB…  ").size(15.0))
                        .clicked()
                    {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("EPUB", &["epub"])
                            .pick_file()
                        {
                            self.open(p);
                        }
                    }
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("o arrastrá un archivo .epub a esta ventana")
                            .size(13.0)
                            .color(th.dim),
                    );

                    let recent = self.cfg.recent.clone();
                    if !recent.is_empty() {
                        ui.add_space(52.0);
                        ui.label(RichText::new("Recientes").size(12.5).color(th.dim));
                        ui.add_space(6.0);
                        for r in recent.iter().take(6) {
                            let name = PathBuf::from(r)
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| r.clone());
                            if ui.link(RichText::new(name).size(14.0)).clicked() {
                                self.open(PathBuf::from(r));
                            }
                        }
                    }
                });
            });
    }

    fn top_bar(&mut self, ctx: &egui::Context, th: &Theme) {
        let (title, author, n) = match &self.book {
            Some(b) => (b.title.clone(), b.author.clone(), b.chapter_count()),
            None => return,
        };
        let chapter_title = self
            .book
            .as_mut()
            .map(|b| b.chapter_title(self.chapter))
            .unwrap_or_default();

        egui::TopBottomPanel::top("bar")
            .exact_height(38.0)
            .frame(
                egui::Frame::new()
                    .fill(th.panel)
                    .inner_margin(egui::Margin::symmetric(10, 6)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui
                        .button(RichText::new("☰").size(14.0))
                        .on_hover_text("Índice (T)")
                        .clicked()
                    {
                        self.show_toc = !self.show_toc;
                    }
                    if ui
                        .button(RichText::new("Aa").size(13.0))
                        .on_hover_text("Tipografía (S)")
                        .clicked()
                    {
                        self.show_settings = !self.show_settings;
                    }
                    ui.add_space(6.0);
                    let shown = if chapter_title.is_empty() {
                        title.clone()
                    } else {
                        format!("{} · {}", title, chapter_title)
                    };
                    let shown = truncate(&shown, 70);
                    ui.label(RichText::new(shown).size(13.5).color(th.fg));
                    if !author.is_empty() {
                        ui.label(
                            RichText::new(truncate(&author, 34))
                                .size(12.0)
                                .color(th.dim),
                        );
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{:.0}%", self.progress() * 100.0))
                                .size(12.5)
                                .color(th.dim),
                        );
                        if ui
                            .button(RichText::new("›").size(16.0))
                            .on_hover_text("Capítulo siguiente (→)")
                            .clicked()
                        {
                            self.goto_chapter(1);
                        }
                        ui.label(
                            RichText::new(format!("{} / {}", self.chapter + 1, n))
                                .size(12.5)
                                .color(th.dim),
                        );
                        if ui
                            .button(RichText::new("‹").size(16.0))
                            .on_hover_text("Capítulo anterior (←)")
                            .clicked()
                        {
                            self.goto_chapter(-1);
                        }
                    });
                });
            });
    }

    fn toc_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_toc {
            return;
        }
        let entries: Vec<(String, usize)> = match &self.book {
            Some(b) => {
                if b.toc.is_empty() {
                    (0..b.chapter_count())
                        .map(|i| (format!("Capítulo {}", i + 1), i))
                        .collect()
                } else {
                    b.toc.iter().map(|e| (e.title.clone(), e.spine)).collect()
                }
            }
            None => return,
        };
        let mut open = self.show_toc;
        let mut jump: Option<usize> = None;
        egui::Window::new("Índice")
            .open(&mut open)
            .resizable(true)
            .default_size([460.0, 540.0])
            .frame(egui::Frame::new().fill(th.panel).inner_margin(12))
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for (title, spine) in &entries {
                        let selected = *spine == self.chapter;
                        let text = RichText::new(title).size(13.5).color(if selected {
                            th.accent
                        } else {
                            th.fg
                        });
                        if ui.selectable_label(selected, text).clicked() {
                            jump = Some(*spine);
                        }
                    }
                });
            });
        self.show_toc = open;
        if let Some(i) = jump {
            self.load_chapter(i, 0.0);
            self.show_toc = false;
        }
    }

    fn settings_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_settings {
            return;
        }
        let mut open = self.show_settings;
        let mut changed = false;
        let s = &mut self.cfg.settings;

        egui::Window::new("Tipografía")
            .open(&mut open)
            .resizable(false)
            .default_width(340.0)
            .frame(egui::Frame::new().fill(th.panel).inner_margin(14))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                ui.label(RichText::new("Tema").size(12.5).color(th.dim));
                ui.horizontal_wrapped(|ui| {
                    for (i, t) in THEMES.iter().enumerate() {
                        let selected = i == s.theme;
                        let (rect, resp) =
                            ui.allocate_exact_size(Vec2::new(50.0, 32.0), Sense::click());
                        ui.painter().rect_filled(rect, 5.0, t.bg);
                        ui.painter().rect_stroke(
                            rect,
                            5.0,
                            Stroke::new(
                                if selected { 2.0 } else { 1.0 },
                                if selected { th.accent } else { th.dim },
                            ),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Aa",
                            FontId::new(15.0, fonts::family(true, false)),
                            t.fg,
                        );
                        if resp.on_hover_text(t.name).clicked() {
                            s.theme = i;
                            changed = true;
                        }
                    }
                });

                ui.add_space(4.0);
                changed |= ui
                    .add(
                        egui::Slider::new(&mut s.font_size, 13.0..=32.0)
                            .text("tamaño")
                            .step_by(0.5),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut s.line_height, 1.2..=2.2)
                            .text("interlineado")
                            .step_by(0.02),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut s.column_width, 420.0..=1100.0)
                            .text("ancho de columna")
                            .step_by(10.0),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut s.paragraph_space, 0.0..=1.5)
                            .text("espacio entre párrafos")
                            .step_by(0.05),
                    )
                    .changed();

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Restablecer").clicked() {
                        let theme_keep = s.theme;
                        *s = Settings::default();
                        s.theme = theme_keep;
                        changed = true;
                    }
                    ui.label(
                        RichText::new("Space / ← → / T / S")
                            .size(11.5)
                            .color(th.dim),
                    );
                });
            });

        self.show_settings = open;
        if changed {
            self.dirty = true;
        }
    }

    fn status_toast(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some((msg, when)) = &self.status else {
            return;
        };
        let msg = msg.clone();
        if when.elapsed() > Duration::from_secs(3) {
            self.status = None;
            return;
        }
        egui::Area::new(egui::Id::new("toast"))
            .anchor(egui::Align2::CENTER_TOP, Vec2::new(0.0, 52.0))
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(Stroke::new(1.0, th.dim.gamma_multiply(0.4)))
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        ui.label(RichText::new(msg).size(13.0).color(th.fg));
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(400));
    }

    // -- lectura -------------------------------------------------------------

    fn reader(&mut self, ctx: &egui::Context, th: &Theme) {
        self.top_bar(ctx, th);

        // Barra de progreso: 2 px al pie.
        egui::TopBottomPanel::bottom("prog")
            .exact_height(2.0)
            .frame(egui::Frame::new().fill(th.bg))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let w = rect.width() * self.progress();
                ui.painter()
                    .rect_filled(rect, 0.0, th.dim.gamma_multiply(0.25));
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.min, Vec2::new(w, rect.height())),
                    0.0,
                    th.accent.gamma_multiply(0.8),
                );
            });

        let offset = (self.scroll + self.delta).max(0.0);
        self.delta = 0.0;

        let mut colw = 660.0;
        let s = self.cfg.settings;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(th.bg))
            .show(ctx, |ui| {
                let avail = ui.available_width();
                colw = s.column_width.min(avail - 48.0).max(240.0);
                let pad = ((avail - colw) * 0.5).max(0.0);

                let Self {
                    blocks,
                    textures,
                    book,
                    scroll,
                    max_scroll,
                    viewport_h,
                    dirty,
                    ..
                } = self;
                let dirty_scroll = dirty;

                let so = ScrollArea::vertical()
                    .id_salt("reader")
                    .auto_shrink([false, false])
                    .vertical_scroll_offset(offset)
                    .show(ui, |ui| {
                        ui.add_space(34.0);
                        ui.horizontal_top(|ui| {
                            ui.add_space(pad);
                            ui.vertical(|ui| {
                                ui.set_min_width(colw);
                                ui.set_max_width(colw);
                                ui.spacing_mut().item_spacing.y = 0.0;
                                for b in blocks.iter() {
                                    draw_block(ui, b, colw, &s, th, textures, book, ctx);
                                }
                            });
                        });
                        ui.add_space(140.0);
                    });

                if (so.state.offset.y - *scroll).abs() > 0.5 {
                    *dirty_scroll = true;
                }
                *scroll = so.state.offset.y;
                *viewport_h = so.inner_rect.height();
                *max_scroll = (so.content_size.y - so.inner_rect.height()).max(0.0);
            });

        // Evita que el layout de la barra se desplace cuando cambia el ancho.
        let _ = colw;
        self.toc_window(ctx, th);
        self.settings_window(ctx, th);
        self.status_toast(ctx, th);

        if self.dirty && self.last_save.elapsed() > Duration::from_millis(700) {
            self.persist();
        }
    }

    fn screenshot(&mut self, ctx: &egui::Context) {
        if self.dev.shot.is_none() {
            return;
        }
        self.shot_frames += 1;

        if self.shot_frames == 12 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }

        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            let w = img.size[0] as u32;
            let h = img.size[1] as u32;
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for px in &img.pixels {
                rgba.extend_from_slice(&[px.r(), px.g(), px.b(), px.a()]);
            }
            if let Some(buf) = image::RgbaImage::from_raw(w, h, rgba) {
                let path = self.dev.shot.clone().unwrap();
                let _ = buf.save(&path);
                eprintln!("screenshot -> {}", path.display());
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ctx.request_repaint();
        }
    }
}

impl eframe::App for Lector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let th = theme::theme(self.cfg.settings.theme);
        if self.applied_theme != Some(self.cfg.settings.theme) {
            th.apply(ctx);
            self.applied_theme = Some(self.cfg.settings.theme);
        }

        self.handle_input(ctx);

        if self.book.is_none() {
            self.open_screen(ctx, &th);
            self.status_toast(ctx, &th);
            if self.dirty && self.last_save.elapsed() > Duration::from_millis(700) {
                self.persist();
            }
        } else {
            self.reader(ctx, &th);
        }

        self.screenshot(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persist();
    }
}

// ---------------------------------------------------------------------------
// Pintado de bloques
// ---------------------------------------------------------------------------

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[allow(clippy::too_many_arguments)]
fn draw_block(
    ui: &mut Ui,
    b: &Block,
    colw: f32,
    s: &Settings,
    th: &Theme,
    textures: &mut HashMap<String, egui::TextureHandle>,
    book: &mut Option<Book>,
    ctx: &egui::Context,
) {
    let base = s.font_size;
    let lh = s.line_height;

    match b.kind {
        Kind::Rule => {
            ui.add_space(base * 0.8);
            ui.horizontal(|ui| {
                ui.add_space(colw * 0.32);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(colw * 0.36, 1.0), Sense::hover());
                ui.painter()
                    .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, th.dim));
            });
            ui.add_space(base * 0.8);
            return;
        }
        Kind::Image => {
            if ensure_texture(ctx, textures, book, b) {
                if let Some(src) = &b.src {
                    if let Some(h) = textures.get(src) {
                        let size = h.size_vec2();
                        let scale = (colw / size.x).min(1.0);
                        let sz = size * scale;
                        ui.add_space(base * 0.7);
                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Image::new(egui::load::SizedTexture::new(h.id(), sz))
                                    .sense(Sense::hover()),
                            );
                        });
                        ui.add_space(base * 0.4);
                    }
                }
            } else if !b.alt.is_empty() {
                ui.add_space(base * 0.4);
                ui.label(
                    RichText::new(b.alt.clone())
                        .size(base * 0.85)
                        .italics()
                        .color(th.dim),
                );
                ui.add_space(base * 0.4);
            }
            return;
        }
        _ => {}
    }

    let (size, force_italic, color, halign, bold_all) = match b.kind {
        Kind::Heading(1) => (base * 1.75, false, th.fg, Align::Min, true),
        Kind::Heading(2) => (base * 1.45, false, th.fg, Align::Min, true),
        Kind::Heading(3) => (base * 1.25, false, th.fg, Align::Min, true),
        Kind::Heading(_) => (base * 1.08, false, th.fg, Align::Min, true),
        Kind::Caption => (base * 0.85, true, th.dim, Align::Center, false),
        Kind::Quote => (
            base * 0.97,
            true,
            th.fg.gamma_multiply(0.92),
            Align::Min,
            false,
        ),
        Kind::Pre => (base * 0.88, false, th.fg, Align::Min, false),
        _ => (base, false, th.fg, Align::Min, false),
    };

    let before = match b.kind {
        Kind::Heading(1) => base * 1.9,
        Kind::Heading(2) => base * 1.7,
        Kind::Heading(_) => base * 1.25,
        Kind::Quote => base * 0.55,
        _ => 0.0,
    };
    let after = match b.kind {
        Kind::Heading(1) => base * 0.75,
        Kind::Heading(_) => base * 0.5,
        Kind::Caption => base * 0.9,
        Kind::Bullet | Kind::Numbered(_) => base * 0.22,
        Kind::Quote => base * 0.4,
        _ => base * s.paragraph_space,
    };

    if before > 0.0 {
        ui.add_space(before);
    }

    let mut job = LayoutJob::default();
    job.wrap.max_width = colw;
    job.wrap.break_anywhere = false;
    job.justify = matches!(
        b.kind,
        Kind::Para | Kind::Quote | Kind::Bullet | Kind::Numbered(_)
    );
    job.halign = halign;

    let section =
        |job: &mut LayoutJob, text: &str, bold: bool, italic: bool, size: f32, color: Color32| {
            let fam = if matches!(b.kind, Kind::Pre) {
                FontFamily::Monospace
            } else {
                fonts::family(bold, italic)
            };
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id: FontId::new(size, fam),
                    color,
                    line_height: Some(size * lh),
                    ..Default::default()
                },
            );
        };

    match b.kind {
        Kind::Bullet => section(
            &mut job,
            "•\u{a0}\u{a0}\u{a0}",
            false,
            false,
            size,
            th.accent,
        ),
        Kind::Numbered(n) => section(
            &mut job,
            &format!("{n}.\u{a0}\u{a0}\u{a0}"),
            false,
            false,
            size,
            th.accent,
        ),
        _ => {}
    }

    for run in &b.runs {
        let rs = if run.small { size * 0.86 } else { size };
        let rc = if run.link { th.accent } else { color };
        section(
            &mut job,
            &run.text,
            run.bold || bold_all,
            run.italic || force_italic,
            rs,
            rc,
        );
    }
    match b.kind {
        Kind::Quote => {
            let x = ui.cursor().left();
            ui.horizontal_top(|ui| {
                ui.add_space(16.0);
                let mut j2 = job.clone();
                j2.wrap.max_width = (colw - 16.0).max(120.0);
                let resp = ui.add(egui::Label::new(j2).wrap().selectable(false));
                ui.painter().vline(
                    x + 3.0,
                    resp.rect.y_range(),
                    Stroke::new(2.0, th.accent.gamma_multiply(0.45)),
                );
            });
        }
        _ => {
            ui.add(egui::Label::new(job).wrap().selectable(false));
        }
    }

    if after > 0.0 {
        ui.add_space(after);
    }
}

fn ensure_texture(
    ctx: &egui::Context,
    textures: &mut HashMap<String, egui::TextureHandle>,
    book: &mut Option<Book>,
    b: &Block,
) -> bool {
    let Some(src) = &b.src else {
        return false;
    };
    if textures.contains_key(src) {
        return true;
    }
    let Some(book) = book.as_mut() else {
        return false;
    };
    let Some(bytes) = book.resource(src) else {
        return false;
    };
    let Ok(img) = image::load_from_memory(&bytes) else {
        return false;
    };
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    if w == 0 || h == 0 {
        return false;
    }
    let color_image = egui::ColorImage::from_rgba_unmultiplied([w, h], rgba.as_ref());
    let handle = ctx.load_texture(src.clone(), color_image, egui::TextureOptions::LINEAR);
    textures.insert(src.clone(), handle);
    true
}
