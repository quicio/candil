//! Ventana y lectura: barra mínima, columna centrada, scroll continuo.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Key, Layout, RichText, ScrollArea, Sense, Stroke, Ui,
    Vec2,
};

use crate::config::{self, Config, Progress, Settings};
use crate::epub::Book;
use crate::fonts;
use crate::library::Library;
use crate::theme::{self, Theme, THEMES};
use crate::xhtml::{Block, Kind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Library,
    Reader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryOrder {
    Recent,
    Title,
    Author,
    Added,
}

impl LibraryOrder {
    fn label(self) -> &'static str {
        match self {
            Self::Recent => "Reciente",
            Self::Title => "Título",
            Self::Author => "Autor",
            Self::Added => "Añadido",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryFilter {
    All,
    Reading,
    Unread,
    Missing,
}

impl LibraryFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "Todos",
            Self::Reading => "Leyendo",
            Self::Unread => "Sin empezar",
            Self::Missing => "Faltantes",
        }
    }
}

struct CardData {
    index: usize,
    path: String,
    title: String,
    author: String,
    cover: Option<(egui::TextureId, Vec2)>,
    chapters: usize,
    chapter: usize,
    missing: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardAction {
    Open,
    Remove,
}

const CARD_W: f32 = 160.0;
const CARD_COVER_H: f32 = 240.0;
const CARD_GAP: f32 = 20.0;
const PAGE_PAD: f32 = 24.0;
const CARD_TEXT_H: f32 = 36.0;
const CARD_AUTHOR_H: f32 = 16.0;
const CARD_STATUS_H: f32 = 24.0;
const CARD_H: f32 = CARD_COVER_H + 8.0 + CARD_TEXT_H + CARD_AUTHOR_H + CARD_STATUS_H;

pub struct Lector {
    screen: Screen,
    book: Option<Book>,
    chapter: usize,
    blocks: Vec<Block>,
    textures: HashMap<String, egui::TextureHandle>,
    library: Library,
    library_textures: HashMap<String, egui::TextureHandle>,
    library_search: String,
    library_order: LibraryOrder,
    library_filter: LibraryFilter,
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
    window_title: String,
}

/// Banderas de desarrollo: captura e inspección visual sin tocar la UI.
#[derive(Debug, Clone, Default)]
pub struct DevOpts {
    pub shot: Option<PathBuf>,
    pub chapter: Option<usize>,
    pub panel: Option<String>,
    /// Reabre el último libro en vez de arrancar en la biblioteca.
    pub resume: bool,
}

impl Lector {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>, dev: DevOpts) -> Self {
        fonts::install(&cc.egui_ctx);
        let (mut cfg, cfg_warning) = Config::load_with_report();
        let (mut library, library_warning) = Library::load_with_report();
        let migrated = library.migrate_progress_from(&mut cfg);
        let mut warning = cfg_warning.or(library_warning);
        if migrated > 0 {
            if let Err(e) = library.save() {
                warning = Some(format!("no pude guardar la biblioteca: {e}"));
            }
            if let Err(e) = cfg.save() {
                warning = Some(format!("no pude guardar la configuración: {e}"));
            }
        }
        // "Recientes" viejo de config.json alimenta el orden sin migrar el archivo.
        let now = crate::library::now();
        for (index, path) in cfg.recent.iter().enumerate() {
            if let Some(entry) = library.books.iter_mut().find(|b| b.path == *path) {
                if entry.last_opened.is_none() {
                    entry.last_opened = Some(now - index as i64);
                }
            }
        }
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
            screen: Screen::Library,
            book: None,
            chapter: 0,
            blocks: Vec::new(),
            textures: HashMap::new(),
            library,
            library_textures: HashMap::new(),
            library_search: String::new(),
            library_order: LibraryOrder::Recent,
            library_filter: LibraryFilter::All,
            cfg,
            scroll: 0.0,
            max_scroll: 0.0,
            delta: 0.0,
            viewport_h: 600.0,
            show_toc: false,
            show_settings: false,
            status: warning.map(|msg| (msg, Instant::now())),
            applied_theme: Some(theme_idx),
            dirty: false,
            last_save: Instant::now(),
            dev,
            shot_frames: 0,
            window_title: "lector".into(),
        };
        app.applied_theme = Some(app.cfg.settings.theme);

        let target = path.or_else(|| {
            if app.dev.resume {
                app.cfg.last_book.clone().map(PathBuf::from)
            } else {
                None
            }
        });
        if let Some(p) = target {
            if p.exists() {
                app.open(p);
            }
        }
        app
    }

    // -- libro ---------------------------------------------------------------

    fn open(&mut self, path: PathBuf) {
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        match Book::open(&path) {
            Ok(book) => {
                let key = book.path.to_string_lossy().to_string();
                let library_progress = self
                    .library
                    .books
                    .iter()
                    .find(|b| b.path == key)
                    .map(|b| b.progress);
                let prog = library_progress
                    .filter(|p| p.chapter > 0 || p.offset > 0.0)
                    .or_else(|| self.cfg.progress.get(&key).copied())
                    .unwrap_or_default();
                let n = book.spine.len();
                self.chapter = prog.chapter.min(n.saturating_sub(1));
                self.book = Some(book);
                self.screen = Screen::Reader;
                self.cfg.last_book = Some(key.clone());
                self.cfg.touch_recent(&key);
                if let Some(entry) = self.library.books.iter_mut().find(|b| b.path == key) {
                    entry.progress = prog;
                    entry.last_opened = Some(crate::library::now());
                    entry.missing = false;
                }
                self.save_library();
                if let Some(ch) = self.dev.chapter {
                    self.chapter = ch.min(n.saturating_sub(1));
                }
                let offset = if self.dev.chapter.is_some() || prog.chapter != self.chapter {
                    0.0
                } else {
                    prog.offset
                };
                self.load_chapter(self.chapter, offset);
                match self.dev.panel.as_deref() {
                    Some("toc") => self.show_toc = true,
                    Some("settings") => self.show_settings = true,
                    _ => {}
                }
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
        let last = n.saturating_sub(1) as i64;
        let next = (cur + index).clamp(0, last) as usize;
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
        if let Some(book) = &self.book {
            let key = book.path.to_string_lossy().to_string();
            let progress = Progress {
                chapter: self.chapter,
                offset: self.scroll,
            };
            if let Some(entry) = self.library.books.iter_mut().find(|b| b.path == key) {
                entry.progress = progress;
                entry.last_opened = Some(crate::library::now());
            }
        }
        self.cfg.settings.clamp();

        let mut ok = true;
        if let Err(e) = self.cfg.save() {
            self.status = Some((
                format!("No pude guardar la configuración: {e}"),
                Instant::now(),
            ));
            ok = false;
        }
        if self.book.is_some() {
            if let Err(e) = self.library.save() {
                self.status = Some((
                    format!("No pude guardar la biblioteca: {e}"),
                    Instant::now(),
                ));
                ok = false;
            }
        }
        self.dirty = !ok;
        self.last_save = Instant::now();
    }

    fn save_library(&mut self) {
        if let Err(e) = self.library.save() {
            self.status = Some((
                format!("No pude guardar la biblioteca: {e}"),
                Instant::now(),
            ));
        }
    }

    fn migrate_progress(&mut self) {
        let migrated = self.library.migrate_progress_from(&mut self.cfg);
        if migrated > 0 {
            self.save_library();
            if let Err(e) = self.cfg.save() {
                self.say(format!("No pude guardar la configuración: {e}"));
            }
        }
    }

    fn say(&mut self, message: impl Into<String>) {
        self.status = Some((message.into(), Instant::now()));
    }

    fn go_library(&mut self) {
        self.persist();
        self.screen = Screen::Library;
        self.book = None;
        self.blocks.clear();
        self.textures.clear();
        self.show_toc = false;
        self.show_settings = false;
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
            if self.show_toc || self.show_settings {
                self.show_toc = false;
                self.show_settings = false;
            } else if self.screen == Screen::Reader {
                self.go_library();
            } else {
                self.library_search.clear();
            }
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
            self.cfg.settings.clamp();
            self.dirty = true;
        }
        if minus {
            self.cfg.settings.font_size -= 1.0;
            self.cfg.settings.clamp();
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

    fn library_screen(&mut self, ctx: &egui::Context, th: &Theme) {
        let mut add_folder = false;
        let mut rescan = false;
        let mut open_path: Option<PathBuf> = None;
        let mut remove: Option<usize> = None;
        let mut remove_folder: Option<String> = None;

        let missing_count = self.library.books.iter().filter(|b| b.missing).count();
        let book_count = self.library.books.len() - missing_count;
        let error_count = self
            .library
            .books
            .iter()
            .filter(|b| b.error.is_some())
            .count();
        let error_details = self
            .library
            .books
            .iter()
            .filter_map(|b| b.error.as_ref().map(|e| format!("{}: {e}", b.title)))
            .collect::<Vec<_>>()
            .join("\n");

        egui::TopBottomPanel::top("library-bar")
            .show_separator_line(false)
            .exact_height(52.0)
            .frame(
                egui::Frame::new()
                    .fill(th.panel)
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new("lector").size(16.0).strong().color(th.fg));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(format!("{book_count} libros"))
                            .size(11.0)
                            .color(th.dim),
                    );
                    if missing_count > 0 {
                        ui.label(
                            RichText::new(format!(
                                "· {missing_count} falta{}",
                                if missing_count == 1 { "" } else { "s" }
                            ))
                            .size(11.0)
                            .color(th.dim),
                        );
                    }
                    if error_count > 0 {
                        let response = ui.label(
                            RichText::new(format!(
                                "· {error_count} error{}",
                                if error_count == 1 { "" } else { "es" }
                            ))
                            .size(11.0)
                            .color(Color32::from_rgb(190, 92, 82)),
                        );
                        if !error_details.is_empty() {
                            response.on_hover_text(error_details.clone());
                        }
                    }
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                    if ui.button("Añadir carpeta…").clicked() {
                        add_folder = true;
                    }
                    if ui.button("Refrescar").clicked() {
                        rescan = true;
                    }
                    ui.add_space(6.0);
                    let response = egui::Frame::new()
                        .fill(th.bg)
                        .corner_radius(7.0)
                        .inner_margin(egui::Margin::symmetric(10, 5))
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.library_search)
                                    .hint_text("Buscar título, autor…")
                                    .desired_width(200.0)
                                    .frame(false),
                            )
                        })
                        .inner;
                    if ctx.input(|i| {
                        (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::F)
                    }) {
                        response.request_focus();
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        egui::ComboBox::from_id_salt("library-filter")
                            .selected_text(self.library_filter.label())
                            .show_ui(ui, |ui| {
                                for filter in [
                                    LibraryFilter::All,
                                    LibraryFilter::Reading,
                                    LibraryFilter::Unread,
                                    LibraryFilter::Missing,
                                ] {
                                    ui.selectable_value(
                                        &mut self.library_filter,
                                        filter,
                                        filter.label(),
                                    );
                                }
                            });
                        ui.label(RichText::new("Filtro").size(11.0).color(th.dim));
                        ui.add_space(4.0);
                        egui::ComboBox::from_id_salt("library-order")
                            .selected_text(self.library_order.label())
                            .show_ui(ui, |ui| {
                                for order in [
                                    LibraryOrder::Recent,
                                    LibraryOrder::Title,
                                    LibraryOrder::Author,
                                    LibraryOrder::Added,
                                ] {
                                    ui.selectable_value(
                                        &mut self.library_order,
                                        order,
                                        order.label(),
                                    );
                                }
                            });
                        ui.label(RichText::new("Orden").size(11.0).color(th.dim));
                    });
                });
            });

        if !self.library.folders.is_empty() {
            let folders = self.library.folders.clone();
            egui::TopBottomPanel::top("library-folders")
                .show_separator_line(false)
                .exact_height(34.0)
                .frame(
                    egui::Frame::new()
                        .fill(th.bg)
                        .inner_margin(egui::Margin::symmetric(16, 6)),
                )
                .show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("Carpetas").size(10.5).color(th.dim));
                        for folder in &folders {
                            let name = Path::new(folder)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| folder.clone());
                            egui::Frame::new()
                                .fill(th.panel)
                                .corner_radius(5.0)
                                .inner_margin(egui::Margin::symmetric(8, 2))
                                .show(ui, |ui| {
                                    ui.horizontal_centered(|ui| {
                                        ui.label(
                                            RichText::new(name.clone()).size(10.5).color(th.fg),
                                        )
                                        .on_hover_text(folder);
                                        let remove_response = ui
                                            .small_button(RichText::new("×").size(11.0))
                                            .on_hover_text("Dejar de vigilar");
                                        if remove_response.clicked() {
                                            remove_folder = Some(folder.clone());
                                        }
                                    });
                                });
                        }
                    });
                });
        }

        let indices = self.visible_books();
        let cards = self.cards(&indices, ctx);
        let empty = self.library.books.is_empty();
        let has_folders = !self.library.folders.is_empty();
        let (empty_title, empty_subtitle) = if has_folders {
            (
                "No hay EPUB en las carpetas vigiladas",
                "Probá Refrescar o añadí otra carpeta.",
            )
        } else {
            (
                "Tu biblioteca está vacía",
                "Añadí una carpeta con archivos .epub.",
            )
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(th.bg))
            .show(ctx, |ui| {
                if empty {
                    ui.vertical_centered(|ui| {
                        ui.add_space(90.0);
                        ui.label(RichText::new(empty_title).size(24.0).color(th.fg));
                        ui.add_space(8.0);
                        ui.label(RichText::new(empty_subtitle).size(13.5).color(th.dim));
                        ui.add_space(22.0);
                        if ui
                            .button(RichText::new("  Añadir carpeta…  ").size(15.0))
                            .clicked()
                        {
                            add_folder = true;
                        }
                        if has_folders {
                            ui.add_space(8.0);
                            if ui
                                .button(RichText::new("  Refrescar  ").size(14.0))
                                .clicked()
                            {
                                rescan = true;
                            }
                        }
                        ui.add_space(8.0);
                        if ui
                            .button(RichText::new("  Abrir un EPUB…  ").size(15.0))
                            .clicked()
                        {
                            if let Some(p) = rfd::FileDialog::new()
                                .add_filter("EPUB", &["epub"])
                                .pick_file()
                            {
                                open_path = Some(p);
                            }
                        }
                        ui.add_space(18.0);
                        ui.label(
                            RichText::new("o arrastrá un archivo .epub a esta ventana")
                                .size(12.5)
                                .color(th.dim),
                        );
                    });
                    return;
                }

                if indices.is_empty() {
                    let message = if self.library_search.trim().is_empty() {
                        "No hay libros en este filtro.".to_string()
                    } else {
                        format!("Nada coincide con «{}»", self.library_search.trim())
                    };
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.label(RichText::new(message).size(18.0).color(th.fg));
                    });
                    return;
                }

                egui::ScrollArea::vertical()
                    .id_salt("library-grid")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(18.0);
                        let avail = ui.available_width();
                        let cols = ((avail - PAGE_PAD * 2.0 + CARD_GAP) / (CARD_W + CARD_GAP))
                            .floor()
                            .max(1.0) as usize;
                        let row_w = cols as f32 * CARD_W + cols.saturating_sub(1) as f32 * CARD_GAP;
                        let left = ((avail - row_w) * 0.5).max(PAGE_PAD);
                        ui.spacing_mut().item_spacing.x = CARD_GAP;

                        for row in cards.chunks(cols) {
                            ui.horizontal_top(|ui| {
                                ui.add_space(left);
                                for card in row {
                                    match library_card(ui, card, th) {
                                        Some(CardAction::Open) => {
                                            open_path = Some(PathBuf::from(&card.path));
                                        }
                                        Some(CardAction::Remove) => remove = Some(card.index),
                                        None => {}
                                    }
                                }
                            });
                            ui.add_space(22.0);
                        }
                        ui.add_space(28.0);
                    });
            });

        if add_folder {
            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                let stats = self.library.scan_folder(&dir);
                self.library_textures.clear();
                self.migrate_progress();
                self.save_library();
                if stats.found == 0 {
                    self.say(format!("No hay EPUB en {}", dir.display()));
                } else {
                    self.say(format!(
                        "{} encontrados · {} nuevos · {} actualizados · {} sin cambios · {} fallos",
                        stats.found, stats.added, stats.updated, stats.unchanged, stats.failed
                    ));
                }
            }
        }
        if rescan {
            let stats = self.library.rescan();
            self.library_textures.clear();
            self.migrate_progress();
            self.save_library();
            self.say(format!(
                "{} encontrados · {} nuevos · {} actualizados · {} sin cambios · {} fallos",
                stats.found, stats.added, stats.updated, stats.unchanged, stats.failed
            ));
        }
        if let Some(folder) = remove_folder {
            self.library.unwatch_folder(&folder);
            self.save_library();
            self.say(format!("Dejé de vigilar {folder}"));
        }
        if let Some(index) = remove {
            if index < self.library.books.len() {
                self.library.books.remove(index);
                self.save_library();
            }
        }
        if let Some(path) = open_path {
            self.open(path);
        }
    }

    fn visible_books(&self) -> Vec<usize> {
        let query = collate(self.library_search.trim());
        let mut out: Vec<usize> = self
            .library
            .books
            .iter()
            .enumerate()
            .filter(|(_, b)| match self.library_filter {
                LibraryFilter::All => true,
                LibraryFilter::Reading => b.progress.chapter > 0 || b.progress.offset > 0.0,
                LibraryFilter::Unread => b.progress.chapter == 0 && b.progress.offset <= 0.0,
                LibraryFilter::Missing => b.missing,
            })
            .filter(|(_, b)| {
                query.is_empty()
                    || collate(&b.title).contains(&query)
                    || collate(&b.author).contains(&query)
                    || collate(&b.path).contains(&query)
            })
            .map(|(index, _)| index)
            .collect();

        match self.library_order {
            LibraryOrder::Recent => out.sort_by(|&a, &b| {
                let ba = &self.library.books[a];
                let bb = &self.library.books[b];
                bb.last_opened
                    .unwrap_or(bb.added)
                    .cmp(&ba.last_opened.unwrap_or(ba.added))
                    .then_with(|| collate(&ba.title).cmp(&collate(&bb.title)))
            }),
            LibraryOrder::Title => out.sort_by(|&a, &b| {
                collate(&self.library.books[a].title).cmp(&collate(&self.library.books[b].title))
            }),
            LibraryOrder::Author => out.sort_by(|&a, &b| {
                let ba = &self.library.books[a];
                let bb = &self.library.books[b];
                collate(&ba.author)
                    .cmp(&collate(&bb.author))
                    .then_with(|| collate(&ba.title).cmp(&collate(&bb.title)))
            }),
            LibraryOrder::Added => out.sort_by(|&a, &b| {
                self.library.books[b]
                    .added
                    .cmp(&self.library.books[a].added)
            }),
        }
        out
    }

    fn cards(&mut self, indices: &[usize], ctx: &egui::Context) -> Vec<CardData> {
        let data_dir = self.library.data_dir.clone();
        indices
            .iter()
            .filter_map(|&index| {
                let entry = self.library.books.get(index)?.clone();
                let cover = entry
                    .cover
                    .as_deref()
                    .and_then(|rel| cover_texture(ctx, &mut self.library_textures, &data_dir, rel));
                Some(CardData {
                    index,
                    path: entry.path,
                    title: entry.title,
                    author: entry.author,
                    cover,
                    chapters: entry.chapters,
                    chapter: entry.progress.chapter,
                    missing: entry.missing,
                    error: entry.error,
                })
            })
            .collect()
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
            .show_separator_line(false)
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
                        egui::Slider::new(&mut s.font_size, config::FONT_MIN..=config::FONT_MAX)
                            .text("tamaño")
                            .step_by(0.5),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut s.line_height, config::LINE_MIN..=config::LINE_MAX)
                            .text("interlineado")
                            .step_by(0.02),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut s.column_width,
                            config::COLUMN_MIN..=config::COLUMN_MAX,
                        )
                        .text("ancho de columna")
                        .step_by(10.0),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut s.paragraph_space,
                            config::PARAGRAPH_MIN..=config::PARAGRAPH_MAX,
                        )
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
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(0.0, -24.0))
            .interactable(false)
            .show(ctx, |ui| {
                let max_w = (ctx.screen_rect().width() - 32.0).clamp(120.0, 620.0);
                ui.set_max_width(max_w);
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
            .show_separator_line(false)
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

        let title = match self.screen {
            Screen::Library => "lector — biblioteca".to_string(),
            Screen::Reader => self
                .book
                .as_ref()
                .map(|book| format!("lector — {}", book.title))
                .unwrap_or_else(|| "lector".to_string()),
        };
        if self.window_title != title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }

        self.handle_input(ctx);

        match self.screen {
            Screen::Library => self.library_screen(ctx, &th),
            Screen::Reader => self.reader(ctx, &th),
        }

        self.status_toast(ctx, &th);

        if self.dirty && self.last_save.elapsed() > Duration::from_millis(700) {
            self.persist();
        }

        self.screenshot(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persist();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let th = theme::theme(self.cfg.settings.theme);
        [
            th.bg.r() as f32 / 255.0,
            th.bg.g() as f32 / 255.0,
            th.bg.b() as f32 / 255.0,
            1.0,
        ]
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

fn collate(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            _ => c,
        })
        .collect()
}

fn pad_to(ui: &mut Ui, top: f32, height: f32) {
    let used = (ui.cursor().top() - top).max(0.0);
    if used < height {
        ui.add_space(height - used);
    }
}

fn cover_texture(
    ctx: &egui::Context,
    cache: &mut HashMap<String, egui::TextureHandle>,
    data_dir: &Path,
    relative: &str,
) -> Option<(egui::TextureId, Vec2)> {
    let path = data_dir.join(relative);
    let key = path.to_string_lossy().to_string();
    if let Some(handle) = cache.get(&key) {
        return Some((handle.id(), handle.size_vec2()));
    }
    let bytes = std::fs::read(&path).ok()?;
    let image = image::load_from_memory(&bytes).ok()?;
    let rgba = image.to_rgba8();
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    if w == 0 || h == 0 {
        return None;
    }
    let color = egui::ColorImage::from_rgba_unmultiplied([w, h], rgba.as_ref());
    let handle = ctx.load_texture(key.clone(), color, egui::TextureOptions::LINEAR);
    let id = handle.id();
    let size = handle.size_vec2();
    cache.insert(key, handle);
    Some((id, size))
}

fn reveal_in_finder(path: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
    }
}

fn library_card(ui: &mut Ui, card: &CardData, th: &Theme) -> Option<CardAction> {
    let mut action = None;
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    let response = ui.allocate_ui_with_layout(
        Vec2::new(CARD_W, CARD_H),
        Layout::top_down(Align::Min),
        |ui| {
            let card_top = ui.cursor().top();
            ui.set_width(CARD_W);
            ui.spacing_mut().item_spacing.y = 0.0;

            let (cover_rect, _) =
                ui.allocate_exact_size(Vec2::new(CARD_W, CARD_COVER_H), Sense::hover());
            ui.painter().rect_filled(cover_rect, 7.0, th.panel);
            if let Some((id, size)) = card
                .cover
                .filter(|(_, size)| size.x >= 32.0 && size.y >= 32.0)
            {
                let scale = ((CARD_W - 10.0) / size.x)
                    .min((CARD_COVER_H - 10.0) / size.y)
                    .min(1.0);
                let shown = size * scale;
                ui.painter().with_clip_rect(cover_rect.shrink(1.0)).image(
                    id,
                    egui::Rect::from_center_size(cover_rect.center(), shown),
                    uv,
                    Color32::WHITE,
                );
            } else {
                let initial = card
                    .title
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                ui.painter().text(
                    cover_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    initial,
                    FontId::new(42.0, fonts::family(true, false)),
                    th.dim,
                );
            }
            if card.missing || card.error.is_some() {
                ui.painter()
                    .rect_filled(cover_rect, 7.0, Color32::from_black_alpha(110));
            }
            ui.painter().rect_stroke(
                cover_rect,
                7.0,
                Stroke::new(1.0, th.dim.gamma_multiply(0.18)),
                egui::StrokeKind::Inside,
            );

            ui.add_space(8.0);
            ui.allocate_ui_with_layout(
                Vec2::new(CARD_W, CARD_TEXT_H),
                Layout::top_down(Align::Min),
                |ui| {
                    let top = ui.cursor().top();
                    ui.set_max_width(CARD_W);
                    ui.add(
                        egui::Label::new(
                            RichText::new(truncate(&card.title, 44))
                                .size(12.5)
                                .color(th.fg),
                        )
                        .wrap(),
                    );
                    pad_to(ui, top, CARD_TEXT_H);
                },
            );
            ui.allocate_ui_with_layout(
                Vec2::new(CARD_W, CARD_AUTHOR_H),
                Layout::top_down(Align::Min),
                |ui| {
                    let top = ui.cursor().top();
                    ui.set_max_width(CARD_W);
                    if !card.author.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(truncate(&card.author, 30))
                                    .size(10.5)
                                    .color(th.dim),
                            )
                            .wrap(),
                        );
                    }
                    pad_to(ui, top, CARD_AUTHOR_H);
                },
            );
            ui.allocate_ui_with_layout(
                Vec2::new(CARD_W, CARD_STATUS_H),
                Layout::top_down(Align::Min),
                |ui| {
                    let top = ui.cursor().top();
                    if card.missing {
                        ui.label(
                            RichText::new("Falta")
                                .size(10.0)
                                .color(Color32::from_rgb(190, 92, 82)),
                        );
                    } else if card.error.is_some() {
                        ui.label(
                            RichText::new("Error al abrir")
                                .size(10.0)
                                .color(Color32::from_rgb(190, 92, 82)),
                        );
                    } else if card.chapters > 0 {
                        let fraction = (card.chapter as f32 / card.chapters as f32).clamp(0.0, 1.0);
                        let (bar, _) =
                            ui.allocate_exact_size(Vec2::new(CARD_W, 2.0), Sense::hover());
                        ui.painter()
                            .rect_filled(bar, 1.0, th.dim.gamma_multiply(0.18));
                        if fraction > 0.0 {
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    bar.min,
                                    Vec2::new(CARD_W * fraction, bar.height()),
                                ),
                                1.0,
                                th.accent.gamma_multiply(0.85),
                            );
                            ui.add_space(3.0);
                            ui.label(
                                RichText::new(format!("{}%", (fraction * 100.0).round() as u32))
                                    .size(9.5)
                                    .color(th.dim),
                            );
                        }
                    }
                    pad_to(ui, top, CARD_STATUS_H);
                },
            );
            pad_to(ui, card_top, CARD_H);
        },
    );

    let rect = response.response.rect;
    let mut response = ui.interact(
        rect,
        ui.id().with(("library-card", card.index)),
        Sense::click(),
    );
    if response.hovered() {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            9.0,
            Stroke::new(1.0, th.accent.gamma_multiply(0.35)),
            egui::StrokeKind::Inside,
        );
    }
    if response.clicked() {
        action = Some(CardAction::Open);
    }
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    let response = if let Some(error) = &card.error {
        response.on_hover_text(error.clone())
    } else if card.missing {
        response.on_hover_text("El archivo ya no está")
    } else {
        response
    };
    response.context_menu(|ui| {
        if ui.button("Mostrar en Finder").clicked() {
            reveal_in_finder(&card.path);
            ui.close();
        }
        if ui.button("Quitar de la biblioteca").clicked() {
            action = Some(CardAction::Remove);
            ui.close();
        }
    });
    action
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
