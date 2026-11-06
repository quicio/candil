//! Ventana y lectura: barra mínima, columna centrada, scroll continuo.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Key, Layout, RichText, ScrollArea, Sense, Stroke, Ui,
    Vec2,
};

use crate::annotations::{Annotation, Annotations};
use crate::config::{self, Config, Progress, Settings};
use crate::epub::Book;
use crate::fonts;
use crate::library::Library;
use crate::search;
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
    ratio: f32,
    missing: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardAction {
    Open,
    Remove,
}

/// Selección activa de texto dentro de un bloque, lista para destacar.
#[derive(Clone, Debug)]
struct Selection {
    chapter: usize,
    block: usize,
    start: usize,
    end: usize,
    quote: String,
    /// Rect en pantalla, para ubicar la barra flotante.
    rect: egui::Rect,
    /// Id del `TextEdit` que la contiene; se usa para soltar el foco.
    edit_id: egui::Id,
}

/// Ventana de nota abierta: crear desde una selección o editar una existente.
#[derive(Clone, Debug)]
struct NoteEditor {
    id: Option<u64>,
    selection: Option<Selection>,
    book: String,
    draft: String,
    color: u8,
    focus: bool,
}

struct AnnRow {
    annotation: Annotation,
    book_title: String,
    chapter_title: String,
    missing: bool,
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
    chapter_anchors: HashMap<String, f32>,
    pending_anchor: Option<String>,
    current_anchor: Option<String>,
    block_positions: Vec<f32>,
    pending_restore: Option<Progress>,
    pending_block_jump: Option<(usize, String)>,
    annotations: Annotations,
    selection: Option<Selection>,
    note: Option<NoteEditor>,
    show_annotations: bool,
    annotations_global: bool,
    active_color: u8,
    show_search: bool,
    search_query: String,
    search_results: Vec<search::Hit>,
    search_truncated: bool,
    search_active: usize,
    search_index: Option<search::SearchIndex>,
    search_focus: bool,
    search_dirty: bool,
    /// Rango del resultado activo: (capítulo, bloque, inicio, fin).
    search_highlight: Option<(usize, usize, usize, usize)>,
    /// Salto único al bloque/offset de un resultado.
    pending_search_jump: Option<(usize, usize)>,
    show_goto: bool,
    goto_percent: f32,
    goto_chapter_number: usize,
    /// Último `TextEdit` del lector con foco (para salir de la selección con Esc).
    reader_focus: Option<egui::Id>,
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
    /// `library` fuerza la pantalla de biblioteca aunque se pase un EPUB.
    pub screen: Option<String>,
    /// Carga `library.json` desde una ruta alternativa (capturas/tests).
    pub library_path: Option<PathBuf>,
    /// Precarga el buscador (capturas).
    pub search: Option<String>,
    /// Precarga la búsqueda dentro del libro (capturas).
    pub book_search: Option<String>,
    /// Salta al primer resultado de la búsqueda (capturas).
    pub search_jump: bool,
    /// Abre el panel de anotaciones en modo global (capturas).
    pub annotations_all: bool,
    /// Exporta las anotaciones a esta ruta sin abrir el diálogo (dev/capturas).
    pub export_md: Option<PathBuf>,
    /// Salta a un ancla del libro al abrir (capturas/dev).
    pub anchor: Option<String>,
}

impl Lector {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>, dev: DevOpts) -> Self {
        fonts::install(&cc.egui_ctx);
        let (mut cfg, cfg_warning) = Config::load_with_report();
        let (annotations, ann_warning) = Annotations::load();
        let (mut library, library_warning) = if let Some(path) = dev.library_path.as_ref() {
            Library::load_with_report_from(path)
        } else {
            Library::load_with_report()
        };
        let migrated = library.migrate_progress_from(&mut cfg);
        let mut warning = cfg_warning.or(library_warning).or(ann_warning);
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
        let initial_search = dev.search.clone().unwrap_or_default();
        let initial_book_search = dev.book_search.clone().unwrap_or_default();
        let mut app = Self {
            screen: Screen::Library,
            book: None,
            chapter: 0,
            blocks: Vec::new(),
            textures: HashMap::new(),
            library,
            library_textures: HashMap::new(),
            library_search: initial_search,
            library_order: LibraryOrder::Recent,
            library_filter: LibraryFilter::All,
            cfg,
            scroll: 0.0,
            max_scroll: 0.0,
            delta: 0.0,
            viewport_h: 600.0,
            show_toc: false,
            show_settings: false,
            chapter_anchors: HashMap::new(),
            pending_anchor: None,
            current_anchor: None,
            block_positions: Vec::new(),
            pending_restore: None,
            pending_block_jump: None,
            annotations,
            selection: None,
            note: None,
            show_annotations: false,
            annotations_global: false,
            active_color: 0,
            show_search: false,
            search_query: initial_book_search.clone(),
            search_results: Vec::new(),
            search_truncated: false,
            search_active: 0,
            search_index: None,
            search_focus: false,
            search_dirty: !initial_book_search.is_empty(),
            search_highlight: None,
            pending_search_jump: None,
            show_goto: false,
            goto_percent: 0.0,
            goto_chapter_number: 1,
            reader_focus: None,
            status: warning.map(|msg| (msg, Instant::now())),
            applied_theme: Some(theme_idx),
            dirty: false,
            last_save: Instant::now(),
            dev,
            shot_frames: 0,
            window_title: "lector".into(),
        };
        app.applied_theme = Some(app.cfg.settings.theme);

        let force_library = app.dev.screen.as_deref() == Some("library");
        let target = if force_library {
            None
        } else {
            path.or_else(|| {
                if app.dev.resume {
                    app.cfg.last_book.clone().map(PathBuf::from)
                } else {
                    None
                }
            })
        };
        if let Some(p) = target {
            if p.exists() {
                app.open(p);
            }
        }
        if app.book.is_some() {
            if let Some(anchor) = app.dev.anchor.clone() {
                let spine = app.book.as_ref().and_then(|book| {
                    book.toc
                        .iter()
                        .find(|entry| entry.anchor.as_deref() == Some(anchor.as_str()))
                        .map(|entry| entry.spine)
                });
                app.jump_to(spine.unwrap_or(app.chapter), Some(anchor));
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
                    .filter(|p| !p.is_empty())
                    .or_else(|| self.cfg.progress.get(&key).copied())
                    .unwrap_or_default();
                let n = book.spine.len();
                self.chapter = prog.chapter.min(n.saturating_sub(1));
                self.book = Some(book);
                self.screen = Screen::Reader;
                self.selection = None;
                self.note = None;
                self.show_annotations = false;
                self.annotations_global = false;
                self.pending_block_jump = None;
                self.reader_focus = None;
                self.search_index = None;
                self.search_results.clear();
                self.search_truncated = false;
                self.search_active = 0;
                self.search_highlight = None;
                self.pending_search_jump = None;
                self.show_goto = false;
                self.search_query = self.dev.book_search.clone().unwrap_or_default();
                self.search_dirty = !self.search_query.is_empty();
                self.show_search = self.dev.book_search.is_some();
                self.search_focus = self.show_search;
                if self.dev.annotations_all {
                    self.annotations_global = true;
                    self.show_annotations = true;
                }
                if let Some(path) = self.dev.export_md.clone() {
                    self.export_annotations_to(&path);
                }
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
                let restore = if self.dev.chapter.is_some() || prog.chapter != self.chapter {
                    None
                } else {
                    Some(prog)
                };
                self.load_chapter(self.chapter, restore);
                match self.dev.panel.as_deref() {
                    Some("toc") => self.show_toc = true,
                    Some("settings") => self.show_settings = true,
                    Some("notes") => self.show_annotations = true,
                    Some("search") => {
                        self.show_search = true;
                        self.search_focus = true;
                        self.search_dirty = !self.search_query.is_empty();
                    }
                    Some("goto") => {
                        self.show_goto = true;
                        self.goto_percent = self.progress() * 100.0;
                        self.goto_chapter_number = self.chapter + 1;
                    }
                    Some("note") => {
                        if let Some(a) = self
                            .annotations
                            .sorted_for_book(&key)
                            .first()
                            .map(|a| (*a).clone())
                        {
                            self.note = Some(NoteEditor {
                                id: Some(a.id),
                                selection: None,
                                book: key.clone(),
                                draft: a.note,
                                color: a.color,
                                focus: true,
                            });
                        }
                    }
                    _ => {}
                }
                self.dirty = true;
            }
            Err(e) => {
                self.status = Some((format!("No pude abrir el archivo: {e}"), Instant::now()));
            }
        }
    }

    fn load_chapter(&mut self, index: usize, restore: Option<Progress>) {
        let blocks = match self.book.as_mut() {
            Some(b) => b.chapter_blocks(index),
            None => Vec::new(),
        };
        self.blocks = blocks;
        self.chapter = index;
        self.textures.clear();
        self.chapter_anchors.clear();
        self.pending_anchor = None;
        self.current_anchor = None;
        self.block_positions.clear();
        self.pending_restore = restore;
        self.pending_block_jump = None;
        self.pending_search_jump = None;
        self.search_highlight = None;
        self.selection = None;
        self.note = None;
        self.reader_focus = None;
        self.scroll = 0.0;
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
            self.load_chapter(next, None);
        }
    }

    fn jump_to(&mut self, spine: usize, anchor: Option<String>) {
        if spine != self.chapter {
            self.load_chapter(spine, None);
        }
        self.pending_restore = None;
        if anchor.is_some() {
            self.pending_anchor = anchor;
        } else {
            self.scroll = 0.0;
            self.pending_anchor = None;
        }
        self.current_anchor = None;
        self.delta = 0.0;
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
            let ratio = if self.max_scroll > 1.0 {
                (self.scroll / self.max_scroll).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let anchor = top_block(self.scroll, &self.block_positions);
            let anchor_ratio = anchor.map(|index| {
                block_fraction(self.scroll, index, &self.block_positions, self.max_scroll)
            });
            let progress = Progress {
                chapter: self.chapter,
                ratio,
                anchor,
                anchor_ratio,
                legacy_offset: None,
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
        self.pending_restore = None;
        self.block_positions.clear();
        self.pending_block_jump = None;
        self.selection = None;
        self.note = None;
        self.show_annotations = false;
        self.annotations_global = false;
        self.reader_focus = None;
        self.show_search = false;
        self.search_index = None;
        self.search_results.clear();
        self.search_truncated = false;
        self.search_active = 0;
        self.search_highlight = None;
        self.pending_search_jump = None;
        self.show_goto = false;
    }

    fn effective_settings(&self) -> Settings {
        let Some(book) = &self.book else {
            return self.cfg.settings;
        };
        let key = book.path.to_string_lossy();
        self.library
            .books
            .iter()
            .find(|entry| entry.path == key)
            .map(|entry| entry.effective_settings(self.cfg.settings))
            .unwrap_or(self.cfg.settings)
    }

    fn has_book_override(&self) -> bool {
        let Some(book) = &self.book else {
            return false;
        };
        let key = book.path.to_string_lossy();
        self.library
            .books
            .iter()
            .find(|entry| entry.path == key)
            .and_then(|entry| entry.settings)
            .is_some()
    }

    fn change_font_size(&mut self, delta: f32) {
        let key = self
            .book
            .as_ref()
            .map(|book| book.path.to_string_lossy().to_string());
        if let Some(key) = key {
            if let Some(entry) = self
                .library
                .books
                .iter_mut()
                .find(|entry| entry.path == key)
            {
                if let Some(settings) = entry.settings.as_mut() {
                    settings.font_size += delta;
                    settings.clamp();
                    self.dirty = true;
                    return;
                }
            }
        }
        self.cfg.settings.font_size += delta;
        self.cfg.settings.clamp();
        self.dirty = true;
    }

    fn book_key(&self) -> Option<String> {
        self.book
            .as_ref()
            .map(|book| book.path.to_string_lossy().to_string())
    }

    fn surrender_reader_focus(&mut self, ctx: &egui::Context) {
        if let Some(id) = self.reader_focus.take() {
            ctx.memory_mut(|memory| memory.surrender_focus(id));
        }
        self.selection = None;
    }

    fn save_annotations(&mut self) {
        if let Err(e) = self.annotations.save() {
            self.status = Some((
                format!("No pude guardar los destacados: {e}"),
                Instant::now(),
            ));
        }
    }

    fn add_highlight(&mut self, ctx: &egui::Context) {
        let Some(selection) = self.selection.take() else {
            return;
        };
        let Some(book) = self.book_key() else {
            return;
        };
        self.annotations.add(Annotation {
            book,
            chapter: selection.chapter,
            block: selection.block,
            start: selection.start,
            end: selection.end,
            quote: selection.quote,
            note: String::new(),
            color: self.active_color.min(2),
            ..Annotation::default()
        });
        self.save_annotations();
        self.reader_focus = Some(selection.edit_id);
        self.surrender_reader_focus(ctx);
        ctx.request_repaint();
    }

    fn open_note_for_selection(&mut self, ctx: &egui::Context) {
        let Some(selection) = self.selection.take() else {
            return;
        };
        let Some(book) = self.book_key() else {
            return;
        };
        self.note = Some(NoteEditor {
            id: None,
            selection: Some(selection.clone()),
            book,
            draft: String::new(),
            color: self.active_color,
            focus: true,
        });
        self.show_search = false;
        self.search_highlight = None;
        self.pending_search_jump = None;
        self.reader_focus = Some(selection.edit_id);
        self.surrender_reader_focus(ctx);
        ctx.request_repaint();
    }

    fn save_note(&mut self) {
        let Some(editor) = self.note.take() else {
            return;
        };
        let draft = editor.draft.trim().to_string();
        if let Some(id) = editor.id {
            self.annotations.update_note(id, &draft);
            self.annotations.update_color(id, editor.color.min(2));
        } else if let Some(selection) = editor.selection {
            self.annotations.add(Annotation {
                book: editor.book,
                chapter: selection.chapter,
                block: selection.block,
                start: selection.start,
                end: selection.end,
                quote: selection.quote,
                note: draft,
                color: editor.color.min(2),
                ..Annotation::default()
            });
        }
        self.save_annotations();
    }

    fn remove_annotation(&mut self, id: u64) {
        if self.annotations.remove(id) {
            self.save_annotations();
        }
    }

    fn jump_to_annotation(&mut self, annotation: &Annotation) {
        if self.book.is_none() {
            return;
        }
        if annotation.chapter != self.chapter {
            self.load_chapter(annotation.chapter, None);
        }
        self.pending_restore = None;
        self.pending_block_jump = Some((annotation.block, annotation.quote.clone()));
        self.show_annotations = false;
        self.selection = None;
        self.note = None;
        self.reader_focus = None;
    }

    fn open_annotation(&mut self, annotation: &Annotation) {
        let target = std::fs::canonicalize(&annotation.book)
            .unwrap_or_else(|_| PathBuf::from(&annotation.book));
        let target_key = target.to_string_lossy().to_string();
        if self.book_key().as_deref() != Some(target_key.as_str()) {
            if !target.is_file() {
                self.status = Some((
                    format!("No encuentro el libro: {}", target.display()),
                    Instant::now(),
                ));
                return;
            }
            self.open(target);
            if self.book_key().as_deref() != Some(target_key.as_str()) {
                return;
            }
        }
        if let Some(a) = self.annotations.get(annotation.id).cloned() {
            self.jump_to_annotation(&a);
        }
    }

    fn export_entries(&mut self) -> Vec<(String, String, Annotation)> {
        let current = self.book_key();
        let items: Vec<Annotation> = if self.annotations_global {
            self.annotations.sorted_all().into_iter().cloned().collect()
        } else if let Some(key) = current.as_deref() {
            self.annotations
                .sorted_for_book(key)
                .into_iter()
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        let mut entries = Vec::with_capacity(items.len());
        for annotation in items {
            let book_title = book_title_for(&self.library, &annotation.book);
            let chapter_title = if current.as_deref() == Some(annotation.book.as_str()) {
                self.book
                    .as_mut()
                    .map(|b| b.chapter_title(annotation.chapter))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            entries.push((book_title, chapter_title, annotation));
        }
        entries
    }

    fn export_annotations(&mut self) {
        let entries = self.export_entries();
        if entries.is_empty() {
            self.status = Some(("No hay destacados para exportar.".into(), Instant::now()));
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("destacados.md")
            .add_filter("Markdown", &["md"])
            .save_file()
        else {
            return;
        };
        self.write_export(&path, &export_markdown(&entries));
    }

    /// Variante para dev/capturas: escribe directo, sin diálogo.
    fn export_annotations_to(&mut self, path: &Path) {
        let entries = self.export_entries();
        if entries.is_empty() {
            self.status = Some(("No hay destacados para exportar.".into(), Instant::now()));
            return;
        }
        self.write_export(path, &export_markdown(&entries));
    }

    fn write_export(&mut self, path: &Path, markdown: &str) {
        match std::fs::write(path, markdown) {
            Ok(()) => {
                self.status = Some((format!("Exportado a {}", path.display()), Instant::now()));
            }
            Err(e) => {
                self.status = Some((format!("No pude exportar: {e}"), Instant::now()));
            }
        }
    }

    fn cycle_search(&mut self, delta: i32, ctx: &egui::Context) {
        let len = self.search_results.len();
        if len == 0 {
            return;
        }
        let next = (self.search_active as i32 + delta).rem_euclid(len as i32) as usize;
        self.activate_search_result(next, ctx);
    }

    fn activate_search_result(&mut self, index: usize, ctx: &egui::Context) {
        let Some(hit) = self.search_results.get(index).cloned() else {
            return;
        };
        self.search_active = index;
        if hit.chapter != self.chapter {
            self.load_chapter(hit.chapter, None);
        }
        self.pending_restore = None;
        self.pending_anchor = None;
        self.pending_block_jump = Some((hit.block, hit.matched.clone()));
        self.pending_search_jump = Some((hit.block, hit.start));
        self.search_highlight = Some((hit.chapter, hit.block, hit.start, hit.end));
        self.selection = None;
        self.note = None;
        self.reader_focus = None;
        ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::new("book-search")));
        ctx.request_repaint();
    }

    fn search_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_search {
            return;
        }

        let query_len = self.search_query.trim().chars().count();
        if query_len >= 2 && self.search_index.is_none() {
            if let Some(book) = self.book.as_mut() {
                self.search_index = Some(search::build(book));
                self.search_dirty = true;
            }
        }
        if self.search_dirty {
            self.search_dirty = false;
            if let Some(index) = self.search_index.as_ref() {
                let (hits, truncated) =
                    search::search(index, &self.search_query, search::MAX_RESULTS);
                self.search_results = hits;
                self.search_truncated = truncated;
                if self.search_active >= self.search_results.len() {
                    self.search_active = 0;
                }
            }
        }

        if self.dev.search_jump && !self.search_results.is_empty() {
            self.dev.search_jump = false;
            let index = self.search_active.min(self.search_results.len() - 1);
            self.activate_search_result(index, ctx);
        }

        let titles: Vec<String> = self
            .search_results
            .iter()
            .map(|hit| {
                self.search_index
                    .as_ref()
                    .and_then(|index| index.chapters.get(hit.chapter))
                    .map(|chapter| chapter.title.clone())
                    .unwrap_or_default()
            })
            .collect();

        let mut open = self.show_search;
        let mut jump = None;
        egui::Window::new("Buscar en el libro")
            .open(&mut open)
            .default_width(460.0)
            .min_width(320.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(th.panel).inner_margin(14))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .id(egui::Id::new("book-search"))
                        .hint_text("Buscar en el libro…")
                        .desired_width(f32::INFINITY),
                );
                if self.search_focus {
                    response.request_focus();
                    self.search_focus = false;
                }
                if response.changed() {
                    self.search_dirty = true;
                    self.search_highlight = None;
                    self.pending_search_jump = None;
                }

                let query_len = self.search_query.trim().chars().count();
                if query_len < 2 {
                    ui.label(
                        RichText::new("Escribí al menos 2 letras.")
                            .size(11.0)
                            .color(th.dim),
                    );
                    return;
                }

                let total = self.search_results.len();
                if total == 0 {
                    ui.label(RichText::new("Sin coincidencias.").size(12.0).color(th.dim));
                    return;
                }

                let count = if self.search_truncated {
                    format!("Mostrando los primeros {total}; afiná la búsqueda.",)
                } else {
                    format!("{total} resultado(s)")
                };
                ui.label(RichText::new(count).size(11.0).color(th.dim));

                let dim = TextFormat {
                    font_id: FontId::new(12.5, fonts::family(false, false)),
                    color: th.dim,
                    ..Default::default()
                };
                let mut accent = dim.clone();
                accent.color = th.accent;

                egui::ScrollArea::vertical()
                    .max_height(380.0)
                    .show(ui, |ui| {
                        for (index, hit) in self.search_results.iter().enumerate() {
                            let mut job = LayoutJob::default();
                            job.wrap.max_width = ui.available_width();
                            job.append(&hit.before, 0.0, dim.clone());
                            job.append(&hit.matched, 0.0, accent.clone());
                            job.append(&hit.after, 0.0, dim.clone());
                            let row = ui.add(egui::Label::new(job).wrap().sense(Sense::click()));
                            if row.clicked() {
                                jump = Some(index);
                            }
                            ui.label(
                                RichText::new(format!(
                                    "Cap. {} · {}",
                                    hit.chapter + 1,
                                    truncate(
                                        titles.get(index).map(String::as_str).unwrap_or(""),
                                        44
                                    )
                                ))
                                .size(10.5)
                                .color(th.dim),
                            );
                            ui.separator();
                        }
                    });

                ui.add_space(2.0);
                ui.label(
                    RichText::new("Enter: ir · n/N: siguiente/anterior · Esc: cerrar")
                        .size(10.5)
                        .color(th.dim),
                );
            });

        self.show_search = open;
        if !open {
            self.search_highlight = None;
            self.pending_search_jump = None;
        }
        if let Some(index) = jump {
            self.activate_search_result(index, ctx);
        }
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
            key_d,
            key_h,
            key_n,
            key_enter,
            shift,
            key_f,
            key_g,
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
                i.key_pressed(Key::D),
                i.key_pressed(Key::H),
                i.key_pressed(Key::N),
                i.key_pressed(Key::Enter),
                i.modifiers.shift,
                i.key_pressed(Key::F),
                i.key_pressed(Key::G),
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

        if ctrl && key_f && self.book.is_some() {
            self.show_search = true;
            self.search_focus = true;
            self.show_toc = false;
            self.show_settings = false;
            self.show_annotations = false;
            self.show_goto = false;
            self.selection = None;
            self.note = None;
            ctx.request_repaint();
        }

        if key_g && !typing && self.book.is_some() && self.note.is_none() {
            self.show_goto = !self.show_goto;
            if self.show_goto {
                self.goto_percent = self.progress() * 100.0;
                self.goto_chapter_number = self.chapter + 1;
                self.show_toc = false;
                self.show_settings = false;
                self.show_search = false;
                self.show_annotations = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
                self.selection = None;
            }
            ctx.request_repaint();
        }

        if key_esc {
            if self.note.is_some() {
                self.note = None;
                ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::new("note-editor")));
                self.surrender_reader_focus(ctx);
            } else if self.show_search {
                self.show_search = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
                ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::new("book-search")));
            } else if self.show_goto {
                self.show_goto = false;
            } else if self.show_annotations {
                self.show_annotations = false;
            } else if self.selection.is_some() || self.reader_focus.is_some() {
                self.surrender_reader_focus(ctx);
            } else if self.show_toc || self.show_settings {
                self.show_toc = false;
                self.show_settings = false;
            } else if self.screen == Screen::Reader {
                self.go_library();
            } else {
                self.library_search.clear();
            }
        }

        if self.book.is_some() && self.note.is_none() {
            if key_d {
                self.show_annotations = !self.show_annotations;
                self.show_toc = false;
                self.show_settings = false;
                if self.show_annotations {
                    self.show_goto = false;
                    self.show_search = false;
                    self.search_highlight = None;
                    self.pending_search_jump = None;
                }
            }
            if self.selection.is_some() {
                if key_h {
                    self.add_highlight(ctx);
                }
                if key_n {
                    self.open_note_for_selection(ctx);
                }
            } else if key_n && !typing && self.show_search && !self.search_results.is_empty() {
                self.cycle_search(if shift { -1 } else { 1 }, ctx);
            }

            if key_enter
                && self.show_search
                && self.note.is_none()
                && !self.search_results.is_empty()
            {
                let index = self.search_active.min(self.search_results.len() - 1);
                self.activate_search_result(index, ctx);
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
            self.change_font_size(1.0);
        }
        if minus {
            self.change_font_size(-1.0);
        }
        if key_t {
            self.show_toc = !self.show_toc;
            self.show_settings = false;
            if self.show_toc {
                self.show_goto = false;
                self.show_search = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
            }
        }
        if key_s {
            self.show_settings = !self.show_settings;
            self.show_toc = false;
            if self.show_settings {
                self.show_goto = false;
                self.show_search = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
            }
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
                LibraryFilter::Reading => b.progress.has_started(),
                LibraryFilter::Unread => !b.progress.has_started(),
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
                    ratio: entry.progress.ratio.clamp(0.0, 1.0),
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
        let book_override = self.has_book_override();

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
                        if self.show_toc {
                            self.show_goto = false;
                            self.show_search = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
                    }
                    if ui
                        .button(RichText::new(if book_override { "Aa·" } else { "Aa" }).size(13.0))
                        .on_hover_text(if book_override {
                            "Tipografía (S) · este libro tiene ajustes propios"
                        } else {
                            "Tipografía (S)"
                        })
                        .clicked()
                    {
                        self.show_settings = !self.show_settings;
                        if self.show_settings {
                            self.show_goto = false;
                            self.show_search = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
                    }
                    if ui
                        .button(RichText::new("Notas").size(12.0))
                        .on_hover_text("Destacados y notas (D)")
                        .clicked()
                    {
                        self.show_annotations = !self.show_annotations;
                        self.show_toc = false;
                        self.show_settings = false;
                        if self.show_annotations {
                            self.show_goto = false;
                            self.show_search = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
                    }
                    if ui
                        .button(RichText::new("Buscar").size(12.0))
                        .on_hover_text("Buscar en el libro (⌘F)")
                        .clicked()
                    {
                        self.show_search = true;
                        self.search_focus = true;
                        self.show_toc = false;
                        self.show_settings = false;
                        self.show_annotations = false;
                        self.show_goto = false;
                        self.selection = None;
                        self.note = None;
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
                        let pct = ui.add(
                            egui::Label::new(
                                RichText::new(format!("{:.0}%", self.progress() * 100.0))
                                    .size(12.5)
                                    .color(th.dim),
                            )
                            .sense(Sense::click()),
                        );
                        if pct.on_hover_text("Ir a… (G)").clicked() {
                            self.show_goto = true;
                            self.goto_percent = self.progress() * 100.0;
                            self.goto_chapter_number = self.chapter + 1;
                            self.show_toc = false;
                            self.show_settings = false;
                            self.show_search = false;
                            self.show_annotations = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
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
        let entries: Vec<(String, usize, Option<String>)> = match &self.book {
            Some(b) => {
                if b.toc.is_empty() {
                    (0..b.chapter_count())
                        .map(|i| (format!("Capítulo {}", i + 1), i, None))
                        .collect()
                } else {
                    b.toc
                        .iter()
                        .map(|e| (e.title.clone(), e.spine, e.anchor.clone()))
                        .collect()
                }
            }
            None => return,
        };
        let mut open = self.show_toc;
        let mut jump: Option<(usize, Option<String>)> = None;
        egui::Window::new("Índice")
            .open(&mut open)
            .resizable(true)
            .default_size([460.0, 540.0])
            .frame(egui::Frame::new().fill(th.panel).inner_margin(12))
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let mut chapter_marked = false;
                    for (title, spine, anchor) in &entries {
                        let selected = if let Some(current) = &self.current_anchor {
                            *spine == self.chapter && anchor.as_deref() == Some(current.as_str())
                        } else {
                            *spine == self.chapter && !chapter_marked
                        };
                        if *spine == self.chapter {
                            chapter_marked = true;
                        }
                        let text = RichText::new(title).size(13.5).color(if selected {
                            th.accent
                        } else {
                            th.fg
                        });
                        if ui.selectable_label(selected, text).clicked() {
                            jump = Some((*spine, anchor.clone()));
                        }
                    }
                });
            });
        self.show_toc = open;
        if let Some((spine, anchor)) = jump {
            self.jump_to(spine, anchor);
            self.show_toc = false;
            ctx.request_repaint();
        }
    }

    fn settings_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_settings {
            return;
        }
        let book_key = self
            .book
            .as_ref()
            .map(|book| book.path.to_string_lossy().to_string());
        let has_book = book_key.is_some();
        let book_index = book_key
            .as_ref()
            .and_then(|key| self.library.books.iter().position(|b| b.path == *key));
        let mut only_book = book_index
            .and_then(|index| self.library.books[index].settings)
            .is_some();
        let mut settings = book_index
            .map(|index| self.library.books[index].effective_settings(self.cfg.settings))
            .unwrap_or(self.cfg.settings);
        let mut open = self.show_settings;
        let mut changed = false;
        let mut toggle_changed = false;

        egui::Window::new("Tipografía")
            .open(&mut open)
            .resizable(false)
            .default_width(340.0)
            .frame(egui::Frame::new().fill(th.panel).inner_margin(14))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                if let Some(_index) = book_index {
                    if ui
                        .checkbox(&mut only_book, "Solo para este libro")
                        .on_hover_text("Guarda tema y tipografía en library.json para este libro")
                        .changed()
                    {
                        toggle_changed = true;
                    }
                    if only_book {
                        ui.label(
                            RichText::new("Ajustes propios de este libro.")
                                .size(10.5)
                                .color(th.dim),
                        );
                    }
                } else if has_book {
                    ui.label(
                        RichText::new(
                            "El libro no está en la biblioteca; los ajustes son globales.",
                        )
                        .size(10.5)
                        .color(th.dim),
                    );
                }

                ui.label(RichText::new("Tema").size(12.5).color(th.dim));
                ui.horizontal_wrapped(|ui| {
                    for (i, t) in THEMES.iter().enumerate() {
                        let selected = i == settings.theme;
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
                            settings.theme = i;
                            changed = true;
                        }
                    }
                });

                ui.add_space(4.0);
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut settings.font_size,
                            config::FONT_MIN..=config::FONT_MAX,
                        )
                        .text("tamaño")
                        .step_by(0.5),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut settings.line_height,
                            config::LINE_MIN..=config::LINE_MAX,
                        )
                        .text("interlineado")
                        .step_by(0.02),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut settings.column_width,
                            config::COLUMN_MIN..=config::COLUMN_MAX,
                        )
                        .text("ancho de columna")
                        .step_by(10.0),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut settings.paragraph_space,
                            config::PARAGRAPH_MIN..=config::PARAGRAPH_MAX,
                        )
                        .text("espacio entre párrafos")
                        .step_by(0.05),
                    )
                    .changed();

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Restablecer").clicked() {
                        let theme_keep = settings.theme;
                        settings = Settings::default();
                        settings.theme = theme_keep;
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
        settings.clamp();

        if toggle_changed {
            if let Some(index) = book_index {
                if only_book {
                    self.library.books[index].settings = Some(settings);
                } else {
                    self.library.books[index].settings = None;
                    settings = self.cfg.settings;
                }
                changed = true;
            }
        }

        if changed {
            if only_book {
                if let Some(index) = book_index {
                    self.library.books[index].settings = Some(settings);
                }
            } else {
                self.cfg.settings = settings;
            }
            self.dirty = true;
        }
    }

    fn selection_bar(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(selection) = self.selection.clone() else {
            return;
        };
        if self.note.is_some() {
            return;
        }
        let screen = ctx.screen_rect();
        let mut pos = egui::pos2(selection.rect.left(), selection.rect.top() - 44.0);
        if pos.y < screen.top() + 44.0 {
            pos.y = selection.rect.bottom() + 8.0;
        }
        pos.x = pos.x.clamp(
            screen.left() + 8.0,
            (screen.right() - 300.0).max(screen.left() + 8.0),
        );

        let mut highlight = false;
        let mut note = false;
        egui::Area::new(egui::Id::new("selection_bar"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(egui::Stroke::new(1.0, th.dim.gamma_multiply(0.45)))
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (index, name) in ["acento", "ámbar", "verde"].iter().enumerate() {
                                ui.selectable_value(
                                    &mut self.active_color,
                                    index as u8,
                                    RichText::new("●")
                                        .size(14.0)
                                        .color(highlight_swatch(th, index as u8)),
                                )
                                .on_hover_text(format!("Color {name}"));
                            }
                            ui.separator();
                            if ui
                                .button(RichText::new("Destacar").size(12.5))
                                .on_hover_text("Destacar selección (H)")
                                .clicked()
                            {
                                highlight = true;
                            }
                            if ui
                                .button(RichText::new("Nota").size(12.5))
                                .on_hover_text("Destacar con nota (N)")
                                .clicked()
                            {
                                note = true;
                            }
                        });
                    });
            });

        if highlight {
            self.add_highlight(ctx);
        }
        if note {
            self.open_note_for_selection(ctx);
        }
    }

    fn annotations_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_annotations {
            return;
        }
        let Some(book_key) = self.book_key() else {
            self.show_annotations = false;
            return;
        };

        let book_items: Vec<Annotation> = self
            .annotations
            .sorted_for_book(&book_key)
            .into_iter()
            .cloned()
            .collect();
        let all_items: Vec<Annotation> =
            self.annotations.sorted_all().into_iter().cloned().collect();
        let current_book_title = book_title_for(&self.library, &book_key);

        let rows_book: Vec<AnnRow> = book_items
            .into_iter()
            .map(|a| {
                let chapter_title = self
                    .book
                    .as_mut()
                    .map(|b| b.chapter_title(a.chapter))
                    .unwrap_or_default();
                let missing =
                    a.chapter == self.chapter && resolve_annotation(&a, &self.blocks).is_none();
                AnnRow {
                    annotation: a,
                    book_title: current_book_title.clone(),
                    chapter_title,
                    missing,
                }
            })
            .collect();
        let rows_all: Vec<AnnRow> = all_items
            .into_iter()
            .map(|a| {
                let is_current = a.book == book_key;
                let chapter_title = if is_current {
                    self.book
                        .as_mut()
                        .map(|b| b.chapter_title(a.chapter))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                let missing = is_current
                    && a.chapter == self.chapter
                    && resolve_annotation(&a, &self.blocks).is_none();
                AnnRow {
                    annotation: a.clone(),
                    book_title: book_title_for(&self.library, &a.book),
                    chapter_title,
                    missing,
                }
            })
            .collect();

        let mut open = self.show_annotations;
        let mut remove = None;
        let mut edit = None;
        let mut jump = None;
        let mut recolor = None;
        let mut export = false;

        egui::Window::new("Destacados y notas")
            .open(&mut open)
            .default_width(410.0)
            .min_width(320.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(th.panel).inner_margin(14))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.annotations_global, false, "Este libro");
                    ui.selectable_value(&mut self.annotations_global, true, "Todos");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(RichText::new("Exportar…").size(11.5))
                            .on_hover_text("Exportar las citas visibles a Markdown")
                            .clicked()
                        {
                            export = true;
                        }
                    });
                });
                ui.separator();

                let rows = if self.annotations_global {
                    &rows_all
                } else {
                    &rows_book
                };

                if rows.is_empty() {
                    let message = if self.annotations_global {
                        "Todavía no hay destacados en ningún libro."
                    } else {
                        "Todavía no destacaste nada."
                    };
                    ui.label(RichText::new(message).size(12.5).color(th.dim));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Seleccioná una frase y usá Destacar o Nota.")
                            .size(11.0)
                            .color(th.dim),
                    );
                    return;
                }

                egui::ScrollArea::vertical()
                    .max_height(430.0)
                    .show(ui, |ui| {
                        for row in rows {
                            let a = &row.annotation;
                            ui.horizontal_top(|ui| {
                                let color_button = ui.add(
                                    egui::Button::new(
                                        RichText::new("●")
                                            .size(13.0)
                                            .color(highlight_swatch(th, a.color)),
                                    )
                                    .frame(false),
                                );
                                if color_button.on_hover_text("Cambiar color").clicked() {
                                    recolor = Some((a.id, (a.color + 1) % 3));
                                }
                                ui.label(
                                    RichText::new(format!("“{}”", truncate(&a.quote, 150)))
                                        .size(13.0)
                                        .color(th.fg),
                                );
                            });
                            if row.missing {
                                ui.label(
                                    RichText::new("(no encontrada en este capítulo)")
                                        .size(11.0)
                                        .color(Color32::from_rgb(190, 92, 82)),
                                );
                            }
                            if !a.note.is_empty() {
                                ui.label(
                                    RichText::new(truncate(&a.note, 220))
                                        .size(12.0)
                                        .italics()
                                        .color(th.fg.gamma_multiply(0.88)),
                                );
                            }
                            ui.horizontal(|ui| {
                                let place = if self.annotations_global {
                                    format!(
                                        "{} · Cap. {}",
                                        truncate(&row.book_title, 32),
                                        a.chapter + 1
                                    )
                                } else {
                                    format!("Cap. {}", a.chapter + 1)
                                };
                                let place = if row.chapter_title.is_empty() {
                                    place
                                } else {
                                    format!("{place} · {}", truncate(&row.chapter_title, 26))
                                };
                                ui.label(RichText::new(place).size(11.0).color(th.dim));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui
                                        .small_button("×")
                                        .on_hover_text("Borrar destacado")
                                        .clicked()
                                    {
                                        remove = Some(a.id);
                                    }
                                    if ui
                                        .small_button("Editar")
                                        .on_hover_text("Editar la nota y el color")
                                        .clicked()
                                    {
                                        edit = Some(a.id);
                                    }
                                    if ui.small_button("Ir").clicked() {
                                        jump = Some(a.id);
                                    }
                                });
                            });
                            ui.separator();
                        }
                    });
            });

        self.show_annotations = open;
        let acted = remove.is_some() || edit.is_some() || jump.is_some() || recolor.is_some();
        if let Some(id) = remove {
            self.remove_annotation(id);
        }
        if let Some((id, color)) = recolor {
            if self.annotations.update_color(id, color) {
                self.save_annotations();
            }
        }
        if let Some(id) = edit {
            if let Some(a) = self.annotations.get(id).cloned() {
                self.note = Some(NoteEditor {
                    id: Some(id),
                    selection: None,
                    book: a.book,
                    draft: a.note,
                    color: a.color,
                    focus: true,
                });
                self.show_search = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
            }
        }
        if let Some(id) = jump {
            if let Some(a) = self.annotations.get(id).cloned() {
                self.open_annotation(&a);
            }
        }
        if export {
            self.export_annotations();
        }
        if acted || export {
            ctx.request_repaint();
        }
    }

    fn note_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(editor) = self.note.as_mut() else {
            return;
        };
        let mut open = true;
        let mut save = false;
        let mut cancel = false;
        let editing = editor.id.is_some();
        let title = if editing { "Editar nota" } else { "Nueva nota" };

        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(420.0)
            .frame(egui::Frame::new().fill(th.panel).inner_margin(14))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Color").size(11.0).color(th.dim));
                    for (index, name) in ["acento", "ámbar", "verde"].iter().enumerate() {
                        ui.selectable_value(
                            &mut editor.color,
                            index as u8,
                            RichText::new("●")
                                .size(14.0)
                                .color(highlight_swatch(th, index as u8)),
                        )
                        .on_hover_text(format!("Color {name}"));
                    }
                });
                ui.label(
                    RichText::new("Nota (opcional; vacía deja solo el destacado)")
                        .size(11.0)
                        .color(th.dim),
                );
                let response = ui.add(
                    egui::TextEdit::multiline(&mut editor.draft)
                        .id(egui::Id::new("note-editor"))
                        .desired_rows(5)
                        .desired_width(f32::INFINITY),
                );
                if editor.focus {
                    response.request_focus();
                    editor.focus = false;
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Guardar").clicked() {
                        save = true;
                    }
                    if ui.button("Cancelar").clicked() {
                        cancel = true;
                    }
                });
            });

        if save {
            self.save_note();
            ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::new("note-editor")));
            ctx.request_repaint();
        } else if cancel || !open {
            self.note = None;
            ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::new("note-editor")));
            ctx.request_repaint();
        }
    }

    fn goto_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_goto {
            return;
        }
        let chapters = self.book.as_ref().map(|b| b.chapter_count()).unwrap_or(0);
        if chapters == 0 {
            self.show_goto = false;
            return;
        }
        let mut open = self.show_goto;
        let mut by_percent = false;
        let mut by_chapter = false;

        egui::Window::new("Ir a")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(340.0)
            .frame(egui::Frame::new().fill(th.panel).inner_margin(14))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Porcentaje del libro")
                        .size(11.0)
                        .color(th.dim),
                );
                ui.add(
                    egui::Slider::new(&mut self.goto_percent, 0.0..=100.0)
                        .suffix(" %")
                        .step_by(0.5),
                );
                if ui.button("Ir al porcentaje").clicked() {
                    by_percent = true;
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("Capítulo (1–{chapters})"))
                        .size(11.0)
                        .color(th.dim),
                );
                ui.add(
                    egui::DragValue::new(&mut self.goto_chapter_number)
                        .range(1..=chapters)
                        .speed(0.2),
                );
                if ui.button("Ir al capítulo").clicked() {
                    by_chapter = true;
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new("G para abrir/cerrar · Esc para cerrar")
                        .size(10.5)
                        .color(th.dim),
                );
            });

        self.show_goto = open;
        if by_percent {
            self.apply_goto_percent();
        }
        if by_chapter {
            self.apply_goto_chapter();
        }
        if !open {
            self.show_goto = false;
        }
    }

    fn apply_goto_percent(&mut self) {
        let Some(chapters) = self.book.as_ref().map(|b| b.chapter_count()) else {
            return;
        };
        let (chapter, fraction) = chapter_for_percent(self.goto_percent, chapters);
        self.load_chapter(
            chapter,
            Some(Progress {
                chapter,
                ratio: fraction,
                anchor: None,
                anchor_ratio: None,
                legacy_offset: None,
            }),
        );
        self.show_goto = false;
    }

    fn apply_goto_chapter(&mut self) {
        let Some(chapters) = self.book.as_ref().map(|b| b.chapter_count()) else {
            return;
        };
        let chapter = self.goto_chapter_number.clamp(1, chapters.max(1)) - 1;
        self.load_chapter(chapter, None);
        self.show_goto = false;
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

        let mut highlights = match self.book_key() {
            Some(key) => {
                resolve_highlights(&self.annotations, &key, self.chapter, &self.blocks, th)
            }
            None => HashMap::new(),
        };
        if let Some((chapter, block, start, end)) = self.search_highlight {
            if chapter == self.chapter {
                highlights
                    .entry(block)
                    .or_default()
                    .push((start, end, search_highlight_color(th)));
            }
        }

        let mut colw = 660.0;
        let s = self.effective_settings();
        let mut link_target: Option<(usize, Option<String>)> = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(th.bg))
            .show(ctx, |ui| {
                let avail = ui.available_width();
                colw = s.column_width.min(avail - 48.0).max(240.0);
                let pad = ((avail - colw) * 0.5).max(0.0);
                let current_chapter = self.chapter;

                let Self {
                    blocks,
                    textures,
                    book,
                    scroll,
                    max_scroll,
                    viewport_h,
                    dirty,
                    chapter_anchors,
                    pending_anchor,
                    current_anchor,
                    block_positions,
                    pending_restore,
                    pending_block_jump,
                    pending_search_jump,
                    selection,
                    reader_focus,
                    ..
                } = self;
                let dirty_scroll = dirty;
                chapter_anchors.clear();
                block_positions.clear();
                *reader_focus = None;

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
                                for (block_index, b) in blocks.iter().enumerate() {
                                    let y = ui.cursor().top();
                                    block_positions.push(y);
                                    if let Some(anchor) = &b.anchor {
                                        chapter_anchors.entry(anchor.clone()).or_insert(y);
                                    }
                                    let ranges = highlights
                                        .get(&block_index)
                                        .map(Vec::as_slice)
                                        .unwrap_or(&[]);
                                    draw_block(
                                        ui,
                                        b,
                                        block_index,
                                        ranges,
                                        colw,
                                        &s,
                                        th,
                                        textures,
                                        book,
                                        current_chapter,
                                        &mut link_target,
                                        selection,
                                        reader_focus,
                                        ctx,
                                    );
                                }
                            });
                        });
                        ui.add_space(140.0);
                    });

                // `ui.cursor()` dentro del ScrollArea da coordenadas de pantalla;
                // el offset de scroll vive en coordenadas del contenido. Sin esta
                // normalización el ancla se guardaba con ~un viewport de error.
                let shift = so.state.offset.y - so.inner_rect.top();
                for y in block_positions.iter_mut() {
                    *y += shift;
                }
                for y in chapter_anchors.values_mut() {
                    *y += shift;
                }

                if (so.state.offset.y - *scroll).abs() > 0.5 {
                    *dirty_scroll = true;
                }
                *scroll = so.state.offset.y;
                *viewport_h = so.inner_rect.height();
                *max_scroll = (so.content_size.y - so.inner_rect.height()).max(0.0);

                if let Some(restore) = pending_restore.take() {
                    // El ancla manda; el ratio solo ajusta dentro del bloque
                    // para recuperar la posición exacta si el layout no cambió.
                    // Sin restar píxeles: así el ancla persistida sigue siendo
                    // el mismo bloque y reabrir no deriva.
                    if let Some(y) = restore_target(&restore, block_positions, *max_scroll) {
                        *scroll = y;
                        ctx.request_repaint();
                    }
                }

                if let Some(anchor) = pending_anchor.take() {
                    if let Some(y) = chapter_anchors.get(&anchor) {
                        *scroll = *y;
                        *current_anchor = Some(anchor);
                        ctx.request_repaint();
                    }
                }

                if let Some((block, quote)) = pending_block_jump.take() {
                    if let Some(index) = locate_quote(blocks, block, &quote) {
                        if let Some(y) = block_positions.get(index) {
                            *scroll = *y;
                            ctx.request_repaint();
                        }
                    }
                }

                if let Some((block, start)) = pending_search_jump.take() {
                    if let Some(&y) = block_positions.get(block) {
                        let next = block_positions
                            .get(block + 1)
                            .copied()
                            .unwrap_or(*max_scroll)
                            .max(y);
                        let len = blocks
                            .get(block)
                            .map(|b| b.text().chars().count())
                            .unwrap_or(0)
                            .max(1);
                        let fraction = (start as f32 / len as f32).clamp(0.0, 1.0);
                        *scroll = (y + fraction * (next - y) - 40.0).clamp(0.0, *max_scroll);
                        ctx.request_repaint();
                    }
                }
            });

        if (self.scroll - offset).abs() > 0.5 {
            self.selection = None;
            self.reader_focus = None;
        }

        if let Some((spine, anchor)) = link_target {
            self.jump_to(spine, anchor);
            ctx.request_repaint();
        }

        // Evita que el layout de la barra se desplace cuando cambia el ancho.
        let _ = colw;
        self.toc_window(ctx, th);
        self.settings_window(ctx, th);
        self.search_window(ctx, th);
        self.goto_window(ctx, th);
        self.selection_bar(ctx, th);
        self.annotations_window(ctx, th);
        self.note_window(ctx, th);
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

/// Traduce un porcentaje del libro a `(capítulo, fracción dentro del capítulo)`.
/// `chapters` debe ser > 0. El 100 % cae en el último capítulo.
fn chapter_for_percent(percent: f32, chapters: usize) -> (usize, f32) {
    if chapters == 0 {
        return (0, 0.0);
    }
    let pos = (percent.clamp(0.0, 100.0) / 100.0 * chapters as f32)
        .clamp(0.0, chapters as f32 - f32::EPSILON);
    let chapter = (pos.floor() as usize).min(chapters - 1);
    let fraction = (pos - chapter as f32).clamp(0.0, 1.0);
    (chapter, fraction)
}

/// Índice del bloque cuyo tope queda en o por encima del scroll.
fn top_block(scroll: f32, positions: &[f32]) -> Option<usize> {
    positions.iter().rposition(|y| *y <= scroll + 1.0)
}

/// Fracción del scroll dentro de un bloque (0..1), para reanclar con precisión
/// incluso en párrafos que ocupan varias pantallas.
fn block_fraction(scroll: f32, index: usize, positions: &[f32], max_scroll: f32) -> f32 {
    let Some(&y) = positions.get(index) else {
        return 0.0;
    };
    let next = positions
        .get(index + 1)
        .copied()
        .unwrap_or(max_scroll)
        .max(y);
    if next <= y {
        return 0.0;
    }
    ((scroll - y) / (next - y)).clamp(0.0, 1.0)
}

fn find_quote(text: &str, quote: &str) -> Option<(usize, usize)> {
    if quote.is_empty() {
        return None;
    }
    let byte = text.find(quote)?;
    let start = text[..byte].chars().count();
    Some((start, start + quote.chars().count()))
}

fn char_slice_eq(text: &str, start: usize, end: usize, quote: &str) -> bool {
    end > start
        && text
            .chars()
            .skip(start)
            .take(end - start)
            .collect::<String>()
            == quote
}

/// Ubica una anotación en el capítulo: primero respeta los offsets guardados;
/// si el bloque o el texto cambiaron, busca la cita. Devuelve (bloque, inicio, fin).
fn resolve_annotation(annotation: &Annotation, blocks: &[Block]) -> Option<(usize, usize, usize)> {
    if let Some(block) = blocks.get(annotation.block) {
        let text = block.text();
        if char_slice_eq(&text, annotation.start, annotation.end, &annotation.quote) {
            return Some((annotation.block, annotation.start, annotation.end));
        }
        if let Some((start, end)) = find_quote(&text, &annotation.quote) {
            return Some((annotation.block, start, end));
        }
    }
    blocks.iter().enumerate().find_map(|(index, block)| {
        find_quote(&block.text(), &annotation.quote).map(|(start, end)| (index, start, end))
    })
}

fn resolve_highlights(
    annotations: &Annotations,
    book: &str,
    chapter: usize,
    blocks: &[Block],
    theme: &Theme,
) -> HashMap<usize, Vec<(usize, usize, Color32)>> {
    let mut out: HashMap<usize, Vec<(usize, usize, Color32)>> = HashMap::new();
    for annotation in annotations.for_book(book).filter(|a| a.chapter == chapter) {
        if let Some((block, start, end)) = resolve_annotation(annotation, blocks) {
            out.entry(block).or_default().push((
                start,
                end,
                highlight_color(theme, annotation.color),
            ));
        }
    }
    out
}

const HIGHLIGHT_AMBER: (u8, u8, u8) = (214, 160, 60);
const HIGHLIGHT_GREEN: (u8, u8, u8) = (74, 150, 110);

/// Color del resaltado con alfa para pintar detrás del texto. `0` usa el acento
/// del tema; `1` ámbar y `2` verde.
fn highlight_color(theme: &Theme, index: u8) -> Color32 {
    let (r, g, b) = match index {
        1 => HIGHLIGHT_AMBER,
        2 => HIGHLIGHT_GREEN,
        _ => (theme.accent.r(), theme.accent.g(), theme.accent.b()),
    };
    Color32::from_rgba_unmultiplied(r, g, b, 90)
}

fn search_highlight_color(theme: &Theme) -> Color32 {
    Color32::from_rgba_unmultiplied(theme.accent.r(), theme.accent.g(), theme.accent.b(), 150)
}

/// Color opaco para los swatches de la UI.
fn highlight_swatch(theme: &Theme, index: u8) -> Color32 {
    match index {
        1 => Color32::from_rgb(HIGHLIGHT_AMBER.0, HIGHLIGHT_AMBER.1, HIGHLIGHT_AMBER.2),
        2 => Color32::from_rgb(HIGHLIGHT_GREEN.0, HIGHLIGHT_GREEN.1, HIGHLIGHT_GREEN.2),
        _ => theme.accent,
    }
}

fn locate_quote(blocks: &[Block], block: usize, quote: &str) -> Option<usize> {
    if quote.is_empty() {
        return (block < blocks.len()).then_some(block);
    }
    if let Some(found) = blocks.get(block) {
        if found.text().contains(quote) {
            return Some(block);
        }
    }
    blocks.iter().position(|b| b.text().contains(quote))
}

/// Destino de restauración: el ancla manda y el ratio ajusta dentro del bloque.
/// Si el ratio se sale del bloque (layout muy distinto) se recorta a sus bordes,
/// de modo que reabrir nunca se aleje más de un bloque del punto guardado.
fn restore_target(restore: &Progress, positions: &[f32], max_scroll: f32) -> Option<f32> {
    if let Some(index) = restore.anchor {
        if let Some(&y) = positions.get(index) {
            let next = positions
                .get(index + 1)
                .copied()
                .unwrap_or(max_scroll)
                .max(y);
            if let Some(fraction) = restore.anchor_ratio {
                let target = y + fraction.clamp(0.0, 1.0) * (next - y);
                return Some(target.clamp(y, next));
            }
            let approx = restore.ratio * max_scroll;
            return Some(approx.clamp(y, next));
        }
    }
    restore
        .legacy_offset
        .map(|offset| offset.max(0.0))
        .or_else(|| (restore.ratio > 0.0).then_some(restore.ratio * max_scroll))
}

impl eframe::App for Lector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme_index = self.effective_settings().theme;
        let th = theme::theme(theme_index);
        if self.applied_theme != Some(theme_index) {
            th.apply(ctx);
            self.applied_theme = Some(theme_index);
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

fn book_title_for(library: &Library, path: &str) -> String {
    library
        .books
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.title.clone())
        .filter(|title| !title.is_empty())
        .or_else(|| {
            Path::new(path)
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| path.to_string())
}

/// Markdown estable para exportar citas: un `##` por libro y un `###` por
/// capítulo. `entries` viene ordenado por libro/capítulo/bloque.
fn export_markdown(entries: &[(String, String, Annotation)]) -> String {
    let mut out = String::from("# Destacados y notas\n\n");
    let mut current_book = "";
    let mut current_chapter: Option<(usize, String)> = None;
    for (book, chapter, annotation) in entries {
        if book != current_book {
            if !current_book.is_empty() {
                out.push('\n');
            }
            out.push_str(&format!("## {book}\n\n"));
            current_book = book;
            current_chapter = None;
        }
        let chapter_label = if chapter.is_empty() {
            format!("Cap. {}", annotation.chapter + 1)
        } else {
            format!("Cap. {} — {chapter}", annotation.chapter + 1)
        };
        let same_chapter = current_chapter
            .as_ref()
            .is_some_and(|(num, label)| *num == annotation.chapter && label == &chapter_label);
        if !same_chapter {
            out.push_str(&format!("### {chapter_label}\n\n"));
            current_chapter = Some((annotation.chapter, chapter_label));
        }
        out.push_str(&format!(
            "> {}\n\n",
            annotation.quote.replace(['\r', '\n'], " ")
        ));
        if !annotation.note.trim().is_empty() {
            out.push_str(annotation.note.trim());
            out.push_str("\n\n");
        }
    }
    out
}

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

fn append_with_highlights(
    job: &mut LayoutJob,
    text: &str,
    format: TextFormat,
    base: usize,
    ranges: &[(usize, usize, Color32)],
) {
    let len = text.chars().count();
    if len == 0 {
        return;
    }
    let overlaps = ranges
        .iter()
        .any(|&(start, end, _)| end > base && start < base + len);
    if !overlaps {
        job.append(text, 0.0, format);
        return;
    }

    let mut cuts = vec![0usize, len];
    for &(start, end, _) in ranges {
        if end <= base || start >= base + len {
            continue;
        }
        cuts.push(start.saturating_sub(base).min(len));
        cuts.push(end.saturating_sub(base).min(len));
    }
    cuts.sort_unstable();
    cuts.dedup();
    let chars: Vec<char> = text.chars().collect();

    for pair in cuts.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        if a >= b {
            continue;
        }
        let abs_a = base + a;
        let abs_b = base + b;
        let mut fmt = format.clone();
        if let Some((_, _, color)) = ranges
            .iter()
            .rev()
            .find(|(start, end, _)| abs_a >= *start && abs_b <= *end)
        {
            fmt.background = *color;
        }
        let segment: String = chars[a..b].iter().collect();
        job.append(&segment, 0.0, fmt);
    }
}

/// Datos de un bloque necesarios para renderizar texto seleccionable.
struct TextArgs<'a> {
    block_index: usize,
    full_text: &'a str,
    block_text: &'a str,
    prefix_len: usize,
    links: &'a [(std::ops::Range<usize>, String)],
    chapter: usize,
    book: Option<&'a Book>,
}

fn selection_range(
    prefix_len: usize,
    block_text: &str,
    a: usize,
    b: usize,
) -> Option<(usize, usize, String)> {
    let (a, b) = (a.min(b), a.max(b));
    let chars = block_text.chars().count();
    let start = a.saturating_sub(prefix_len).min(chars);
    let end = b.saturating_sub(prefix_len).min(chars);
    (end > start).then(|| {
        let quote = block_text.chars().skip(start).take(end - start).collect();
        (start, end, quote)
    })
}

fn add_selectable_text(
    ui: &mut Ui,
    job: LayoutJob,
    args: &TextArgs<'_>,
    link_target: &mut Option<(usize, Option<String>)>,
    selection: &mut Option<Selection>,
    reader_focus: &mut Option<egui::Id>,
) -> egui::Response {
    let desired_width = job.wrap.max_width;
    let wrap_width = desired_width;
    let mut layouter = move |ui: &egui::Ui, _buf: &dyn egui::TextBuffer, wrap_width: f32| {
        let mut job = job.clone();
        job.wrap.max_width = wrap_width;
        ui.fonts(|fonts| fonts.layout_job(job))
    };
    let id = ui.make_persistent_id((
        "reader-block",
        args.chapter,
        args.block_index,
        args.book.map(|b| b.path.as_path()),
    ));
    let mut text = args.full_text;
    let output = egui::TextEdit::multiline(&mut text)
        .id(id)
        .frame(false)
        .margin(egui::Margin::ZERO)
        .desired_rows(1)
        .desired_width(wrap_width)
        .layouter(&mut layouter)
        .show(ui);
    let response = output.response;

    if response.has_focus() {
        *reader_focus = Some(response.id);
    }

    if let Some(range) = output.cursor_range {
        let (a, b) = (range.primary.index, range.secondary.index);
        let (a, b) = (a.min(b), a.max(b));
        let focused = response.has_focus();
        if focused && b == a {
            *selection = None;
        } else if focused {
            if let Some((start, end, quote)) =
                selection_range(args.prefix_len, args.block_text, a, b)
            {
                let start_rect = output.galley.pos_from_cursor(egui::text::CCursor::new(a));
                let end_rect = output.galley.pos_from_cursor(egui::text::CCursor::new(b));
                let rect = egui::Rect::from_two_pos(
                    output.galley_pos + start_rect.left_top().to_vec2(),
                    output.galley_pos + end_rect.right_bottom().to_vec2(),
                );
                *selection = Some(Selection {
                    chapter: args.chapter,
                    block: args.block_index,
                    start,
                    end,
                    quote,
                    rect,
                    edit_id: response.id,
                });
            } else {
                *selection = None;
            }
        }
    }

    let collapsed = output
        .cursor_range
        .is_none_or(|range| range.primary.index == range.secondary.index);
    if response.has_focus() && collapsed && ui.input(|i| i.pointer.any_released()) {
        ui.memory_mut(|memory| memory.surrender_focus(response.id));
        *reader_focus = None;
    }

    if !args.links.is_empty() && response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let index = output.galley.cursor_from_pos(pos - output.galley_pos).index;
            if let Some((_, href)) = args.links.iter().find(|(range, _)| range.contains(&index)) {
                if let Some(book) = args.book {
                    if let Some(target) = book.link_target(href, args.chapter) {
                        *link_target = Some(target);
                    }
                }
            }
        }
    }
    response
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
                        let fraction = ((card.chapter as f32 + card.ratio) / card.chapters as f32)
                            .clamp(0.0, 1.0);
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
    block_index: usize,
    highlights: &[(usize, usize, Color32)],
    colw: f32,
    s: &Settings,
    th: &Theme,
    textures: &mut HashMap<String, egui::TextureHandle>,
    book: &mut Option<Book>,
    chapter: usize,
    link_target: &mut Option<(usize, Option<String>)>,
    selection: &mut Option<Selection>,
    reader_focus: &mut Option<egui::Id>,
    ctx: &egui::Context,
) {
    let base = s.font_size;
    let lh = s.line_height;

    // Bloques-ancla invisibles (`<a id="…"></a>`): no ocupan espacio.
    if b.runs.is_empty() && b.src.is_none() && !matches!(b.kind, Kind::Rule | Kind::Image) {
        return;
    }

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

    let mut cursor = 0usize;
    let mut text_cursor = 0usize;
    let mut link_spans: Vec<(std::ops::Range<usize>, String)> = Vec::new();

    let format_for = |bold: bool, italic: bool, size: f32, color: Color32| -> TextFormat {
        let fam = if matches!(b.kind, Kind::Pre) {
            FontFamily::Monospace
        } else {
            fonts::family(bold, italic)
        };
        TextFormat {
            font_id: FontId::new(size, fam),
            color,
            line_height: Some(size * lh),
            ..Default::default()
        }
    };

    match b.kind {
        Kind::Bullet => {
            let prefix = "•\u{a0}\u{a0}\u{a0}";
            job.append(prefix, 0.0, format_for(false, false, size, th.accent));
            cursor += prefix.chars().count();
        }
        Kind::Numbered(n) => {
            let prefix = format!("{n}.\u{a0}\u{a0}\u{a0}");
            job.append(&prefix, 0.0, format_for(false, false, size, th.accent));
            cursor += prefix.chars().count();
        }
        _ => {}
    }
    let prefix_len = cursor;

    for run in &b.runs {
        let rs = if run.small { size * 0.86 } else { size };
        let rc = if run.link { th.accent } else { color };
        let len = run.text.chars().count();
        if let Some(href) = &run.href {
            link_spans.push((cursor..cursor + len, href.clone()));
        }
        append_with_highlights(
            &mut job,
            &run.text,
            format_for(run.bold || bold_all, run.italic || force_italic, rs, rc),
            text_cursor,
            highlights,
        );
        cursor += len;
        text_cursor += len;
    }

    let full_text = job.text.clone();
    let block_text = b.text();
    match b.kind {
        Kind::Quote => {
            let x = ui.cursor().left();
            ui.horizontal_top(|ui| {
                ui.add_space(16.0);
                let mut j2 = job.clone();
                j2.wrap.max_width = (colw - 16.0).max(120.0);
                let args = TextArgs {
                    block_index,
                    full_text: &full_text,
                    block_text: &block_text,
                    prefix_len,
                    links: &link_spans,
                    chapter,
                    book: book.as_ref(),
                };
                let resp = add_selectable_text(ui, j2, &args, link_target, selection, reader_focus);
                ui.painter().vline(
                    x + 3.0,
                    resp.rect.y_range(),
                    Stroke::new(2.0, th.accent.gamma_multiply(0.45)),
                );
            });
        }
        _ => {
            let args = TextArgs {
                block_index,
                full_text: &full_text,
                block_text: &block_text,
                prefix_len,
                links: &link_spans,
                chapter,
                book: book.as_ref(),
            };
            add_selectable_text(ui, job, &args, link_target, selection, reader_focus);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn prog(ratio: f32, anchor: Option<usize>) -> Progress {
        Progress {
            chapter: 3,
            ratio,
            anchor,
            anchor_ratio: None,
            legacy_offset: None,
        }
    }

    #[test]
    fn ancla_no_deriva_al_reabrir() {
        let positions = [34.0, 120.0, 300.0, 700.0];
        let p = prog(0.3, Some(2));
        let target = restore_target(&p, &positions, 1000.0).unwrap();
        assert!((target - 300.0).abs() < 0.01);
        assert_eq!(top_block(target, &positions), Some(2));
    }

    #[test]
    fn fraccion_del_bloque_sobrevive_al_cambio_de_layout() {
        let p = Progress {
            chapter: 3,
            ratio: 0.25,
            anchor: Some(2),
            anchor_ratio: Some(0.5),
            legacy_offset: None,
        };
        let antes = restore_target(&p, &[34.0, 120.0, 300.0, 700.0], 1000.0).unwrap();
        assert!((antes - 500.0).abs() < 0.01);

        // El mismo 50 % del bloque cae en otro píxel tras cambiar la fuente.
        let despues = restore_target(&p, &[34.0, 150.0, 400.0, 1200.0], 2000.0).unwrap();
        assert!((despues - 800.0).abs() < 0.01);
    }

    #[test]
    fn fraccion_cero_fija_el_tope_del_bloque() {
        let p = Progress {
            chapter: 3,
            ratio: 0.9,
            anchor: Some(2),
            anchor_ratio: Some(0.0),
            legacy_offset: None,
        };
        let target = restore_target(&p, &[34.0, 120.0, 300.0, 700.0], 1000.0).unwrap();
        assert!((target - 300.0).abs() < 0.01);
    }

    #[test]
    fn fraccion_se_calcula_dentro_del_bloque() {
        let positions = [34.0, 120.0, 300.0, 700.0];
        assert!((block_fraction(500.0, 2, &positions, 1000.0) - 0.5).abs() < 0.01);
        assert!((block_fraction(300.0, 2, &positions, 1000.0)).abs() < 0.01);
        assert!((block_fraction(700.0, 2, &positions, 1000.0) - 1.0).abs() < 0.01);
        assert!((block_fraction(850.0, 3, &positions, 1000.0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn ratio_ajusta_dentro_del_ancla() {
        let positions = [34.0, 120.0, 300.0, 700.0];
        let dentro = restore_target(&prog(0.5, Some(2)), &positions, 1000.0).unwrap();
        assert!((dentro - 500.0).abs() < 0.01);

        let debajo = restore_target(&prog(1.0, Some(2)), &positions, 1000.0).unwrap();
        assert!((debajo - 700.0).abs() < 0.01);

        let encima = restore_target(&prog(0.05, Some(2)), &positions, 1000.0).unwrap();
        assert!((encima - 300.0).abs() < 0.01);
    }

    #[test]
    fn sin_ancla_cae_a_ratio_o_al_offset_viejo() {
        let positions = [34.0, 120.0];
        let ratio = restore_target(&prog(0.25, None), &positions, 800.0).unwrap();
        assert!((ratio - 200.0).abs() < 0.01);

        let viejo = Progress {
            chapter: 1,
            ratio: 0.0,
            anchor: None,
            anchor_ratio: None,
            legacy_offset: Some(42.0),
        };
        assert_eq!(restore_target(&viejo, &positions, 800.0), Some(42.0));

        let fuera = restore_target(&prog(0.1, Some(99)), &positions, 800.0).unwrap();
        assert!((fuera - 80.0).abs() < 0.01);
    }

    fn paragraph(text: &str) -> Block {
        Block {
            kind: Kind::Para,
            runs: vec![crate::xhtml::Run {
                text: text.into(),
                ..Default::default()
            }],
            src: None,
            alt: String::new(),
            anchor: None,
        }
    }

    fn annotation(block: usize, start: usize, end: usize, quote: &str) -> Annotation {
        Annotation {
            book: "/libro.epub".into(),
            chapter: 2,
            block,
            start,
            end,
            quote: quote.into(),
            ..Annotation::default()
        }
    }

    #[test]
    fn destacado_respeta_offsets_y_reancla_por_cita() {
        let blocks = vec![paragraph("Hola mundo"), paragraph("Otra frase distinta")];
        let exacto = annotation(0, 5, 10, "mundo");
        assert_eq!(resolve_annotation(&exacto, &blocks), Some((0, 5, 10)));

        // El bloque guardado cambió, pero la cita sigue en el capítulo.
        let movido = annotation(1, 0, 5, "mundo");
        assert_eq!(resolve_annotation(&movido, &blocks), Some((0, 5, 10)));

        // Offsets viejos dentro del bloque correcto: se reancla por cita.
        let corrido = annotation(0, 0, 9, "frase distinta");
        assert_eq!(resolve_annotation(&corrido, &blocks), Some((1, 5, 19)));
    }

    #[test]
    fn destacado_inexistente_no_se_dibuja() {
        let blocks = vec![paragraph("Hola mundo")];
        let th = theme::theme(0);
        let perdido = annotation(0, 0, 4, "zzz");
        assert_eq!(resolve_annotation(&perdido, &blocks), None);

        let mut all = Annotations::default();
        all.add(perdido);
        all.add(annotation(0, 5, 10, "mundo"));
        let ranges = resolve_highlights(&all, "/libro.epub", 2, &blocks, &th);
        let ranges = ranges.get(&0).expect("un rango válido");
        assert_eq!(ranges.len(), 1);
        assert_eq!((ranges[0].0, ranges[0].1), (5, 10));
        assert_eq!(ranges[0].2, highlight_color(&th, 0));
    }

    #[test]
    fn colores_distintos_conviven_en_el_mismo_bloque() {
        let blocks = vec![paragraph("uno dos tres")];
        let th = theme::theme(0);
        let mut all = Annotations::default();
        let mut a1 = annotation(0, 0, 3, "uno");
        a1.color = 1;
        let mut a2 = annotation(0, 8, 12, "tres");
        a2.color = 2;
        all.add(a1);
        all.add(a2);

        let ranges = resolve_highlights(&all, "/libro.epub", 2, &blocks, &th);
        let ranges = ranges.get(&0).expect("dos rangos");
        assert_eq!(ranges.len(), 2);
        assert_ne!(ranges[0].2, ranges[1].2);
        assert_eq!(ranges[0].2, highlight_color(&th, 1));
        assert_eq!(ranges[1].2, highlight_color(&th, 2));
    }

    #[test]
    fn locate_quote_prefiere_el_bloque_guardado() {
        let blocks = vec![paragraph("Hola mundo"), paragraph("mundo otra vez")];
        assert_eq!(locate_quote(&blocks, 1, "mundo"), Some(1));
        assert_eq!(locate_quote(&blocks, 9, "mundo"), Some(0));
        assert_eq!(locate_quote(&blocks, 0, "zzz"), None);
        assert_eq!(locate_quote(&blocks, 1, ""), Some(1));
    }

    #[test]
    fn seleccion_descuenta_prefijo_y_ordena_extremos() {
        assert_eq!(
            selection_range(0, "hola mundo", 0, 4),
            Some((0, 4, "hola".to_string()))
        );
        assert_eq!(
            selection_range(4, "hola mundo", 4, 9),
            Some((0, 5, "hola ".to_string()))
        );
        assert_eq!(
            selection_range(4, "hola mundo", 9, 4),
            Some((0, 5, "hola ".to_string()))
        );
        assert_eq!(selection_range(4, "hola mundo", 2, 3), None);
        assert_eq!(
            selection_range(0, "hola", 0, 99),
            Some((0, 4, "hola".to_string()))
        );
    }

    #[test]
    fn porcentaje_a_capitulo() {
        assert_eq!(chapter_for_percent(0.0, 10), (0, 0.0));
        let (chapter, fraction) = chapter_for_percent(50.0, 10);
        assert_eq!(chapter, 5);
        assert!((fraction - 0.0).abs() < 0.01);
        let (chapter, fraction) = chapter_for_percent(100.0, 10);
        assert_eq!(chapter, 9);
        assert!(fraction > 0.99);
        assert_eq!(chapter_for_percent(99.999, 10).0, 9);
        assert_eq!(chapter_for_percent(50.0, 0), (0, 0.0));
    }

    #[test]
    fn export_markdown_agrupa_libros_y_conserva_citas() {
        let a1 = Annotation {
            chapter: 2,
            quote: "uno\ndos".into(),
            note: "nota".into(),
            ..Annotation::default()
        };
        let a2 = Annotation {
            chapter: 0,
            quote: "tres".into(),
            ..Annotation::default()
        };
        let a3 = Annotation {
            chapter: 2,
            quote: "cuatro".into(),
            ..Annotation::default()
        };
        let entries = vec![
            ("Libro A".to_string(), "Capítulo X".to_string(), a1),
            ("Libro A".to_string(), "Capítulo X".to_string(), a3),
            ("Libro B".to_string(), String::new(), a2),
        ];
        let md = export_markdown(&entries);
        assert!(md.starts_with("# Destacados y notas"));
        assert!(md.contains("## Libro A"));
        assert!(md.contains("### Cap. 3 — Capítulo X"));
        assert_eq!(md.matches("### Cap. 3 — Capítulo X").count(), 1);
        assert!(md.contains("> uno dos"));
        assert!(md.contains("> cuatro"));
        assert!(md.contains("nota"));
        assert!(md.contains("## Libro B"));
        assert!(md.contains("### Cap. 1\n"));
    }

    #[test]
    fn rangos_solapados_gana_el_ultimo_color() {
        let mut job = LayoutJob::default();
        let first = Color32::RED;
        let second = Color32::GREEN;
        append_with_highlights(
            &mut job,
            "abcdef",
            TextFormat::default(),
            0,
            &[(0, 4, first), (3, 6, second)],
        );
        assert_eq!(job.sections.len(), 3);
        assert_eq!(job.sections[0].format.background, first);
        assert_eq!(job.sections[1].format.background, second);
        assert_eq!(job.sections[2].format.background, second);
        assert_eq!(job.text, "abcdef");
    }
}
