//! Ventana y lectura: barra mínima, columna centrada, scroll continuo.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{
    self, Align, Color32, FontFamily, FontId, Key, Layout, RichText, ScrollArea, Sense, Stroke, Ui,
    Vec2,
};

use crate::ai;
use crate::annotations::{Annotation, Annotations};
use crate::backup;
use crate::config::{self, Config, Progress, Settings};
use crate::consulta::{
    self, AiPrompt, ConsultaEntry, ConsultaHistory, ConsultaKind, ConsultaRequest, ConsultaResponse,
};
use crate::epub::Book;
use crate::epub_edit::{self, CoverExt, CoverUpload, Edits};
use crate::fonts;
use crate::library::{BookEntry, Bookmark, Library, ScanStats};
use crate::search;
use crate::secrets;
use crate::theme::{self, Theme, THEMES};
use crate::ui as widgets;
use crate::update;
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
    Series,
    Added,
}

impl LibraryOrder {
    fn label(self) -> &'static str {
        match self {
            Self::Recent => "Reciente",
            Self::Title => "Título",
            Self::Author => "Autor",
            Self::Series => "Serie",
            Self::Added => "Añadido",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LibraryFilter {
    All,
    Reading,
    Unread,
    Missing,
    Duplicates,
    Series(String),
    NoSeries,
}

impl LibraryFilter {
    fn label(&self) -> String {
        match self {
            Self::All => "Todos".into(),
            Self::Reading => "Leyendo".into(),
            Self::Unread => "Sin empezar".into(),
            Self::Missing => "Faltantes".into(),
            Self::Duplicates => "Duplicados".into(),
            Self::Series(name) => {
                let short: String = name.chars().take(22).collect();
                if short.chars().count() < name.chars().count() {
                    format!("Serie · {short}…")
                } else {
                    format!("Serie · {short}")
                }
            }
            Self::NoSeries => "Sin serie".into(),
        }
    }
}

/// Clave de posible duplicado: título y autor normalizados. No se usa el
/// contenido ni el nombre de archivo; dos ediciones distintas pueden compartir
/// título y autor, por eso la UI habla de "posibles duplicados".
fn duplicate_key(book: &BookEntry) -> Option<String> {
    let title = collate(book.title.trim());
    if title.is_empty() {
        return None;
    }
    let author = collate(book.author.trim());
    Some(format!("{title}\u{1f}{author}"))
}

fn duplicate_counts(books: &[BookEntry]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for book in books {
        if let Some(key) = duplicate_key(book) {
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts
}

fn matches_filter(
    filter: &LibraryFilter,
    book: &BookEntry,
    duplicates: &HashMap<String, usize>,
) -> bool {
    match filter {
        LibraryFilter::All => true,
        LibraryFilter::Reading => book.progress.has_started(),
        LibraryFilter::Unread => !book.progress.has_started(),
        LibraryFilter::Missing => book.missing,
        LibraryFilter::Duplicates => duplicate_key(book)
            .is_some_and(|key| duplicates.get(&key).is_some_and(|count| *count > 1)),
        LibraryFilter::Series(name) => book
            .series
            .as_deref()
            .is_some_and(|series| series.trim().eq_ignore_ascii_case(name.trim())),
        LibraryFilter::NoSeries => book
            .series
            .as_deref()
            .is_none_or(|name| name.trim().is_empty()),
    }
}

const GRID_VIRTUALIZE_AFTER: usize = 240;
const GRID_TOP_PAD: f32 = 18.0;
const GRID_BOTTOM_PAD: f32 = 28.0;
const GRID_ROW_GAP: f32 = 22.0;
const GRID_HEADER_H: f32 = 34.0;

#[derive(Debug, Clone, PartialEq, Eq)]
enum GridItem {
    Header { label: String, count: usize },
    Row { start: usize, end: usize },
}

impl GridItem {
    fn height(&self) -> f32 {
        match self {
            Self::Header { .. } => GRID_HEADER_H,
            Self::Row { .. } => CARD_H + GRID_ROW_GAP,
        }
    }
}

fn build_grid_items(
    series: &[Option<String>],
    cols: usize,
    group_by_series: bool,
) -> Vec<GridItem> {
    let cols = cols.max(1);
    let mut items = Vec::new();
    if !group_by_series {
        for start in (0..series.len()).step_by(cols) {
            items.push(GridItem::Row {
                start,
                end: (start + cols).min(series.len()),
            });
        }
        return items;
    }

    let mut position = 0;
    while position < series.len() {
        let name = series[position].clone();
        let mut end = position + 1;
        while end < series.len() {
            let same = match (name.as_deref(), series[end].as_deref()) {
                (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
                (None, None) => true,
                _ => false,
            };
            if !same {
                break;
            }
            end += 1;
        }
        items.push(GridItem::Header {
            label: name.unwrap_or_else(|| "Sin serie".into()),
            count: end - position,
        });
        for start in (position..end).step_by(cols) {
            items.push(GridItem::Row {
                start,
                end: (start + cols).min(end),
            });
        }
        position = end;
    }
    items
}

const CONTENT_SEARCH_MIN: usize = 3;
const CONTENT_SEARCH_MAX_HITS: usize = 100;
const CONTENT_SEARCH_PER_BOOK: usize = 20;
const CONTENT_SEARCH_MAX_BOOKS: usize = 120;

#[derive(Debug, Clone)]
struct LibraryContentHit {
    path: PathBuf,
    book_title: String,
    chapter: usize,
    chapter_title: String,
    block: usize,
    start: usize,
    end: usize,
    before: String,
    matched: String,
    after: String,
}

enum LibraryContentMsg {
    Progress {
        scanned: usize,
        total: usize,
        failed: usize,
    },
    Hits(Vec<LibraryContentHit>),
    Done {
        scanned: usize,
        total: usize,
        failed: usize,
        truncated: bool,
    },
}

struct LibraryContentSearch {
    query: String,
    rx: Receiver<LibraryContentMsg>,
    cancel: Arc<AtomicBool>,
    hits: Vec<LibraryContentHit>,
    scanned: usize,
    total: usize,
    failed: usize,
    busy: bool,
    truncated: bool,
}

struct CardData {
    index: usize,
    path: String,
    title: String,
    author: String,
    series: Option<String>,
    series_name: Option<String>,
    /// Cantidad de libros con el mismo título y autor; 0 = único.
    duplicate_count: usize,
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
    Locate,
    Recover,
    Edit,
}

/// Un extremo de la selección: bloque y offset de caracteres dentro de
/// `Block::text()` (sin contar el prefijo de viñeta/numeración).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Anchor {
    block: usize,
    offset: usize,
}

impl Anchor {
    fn key(self) -> (usize, usize) {
        (self.block, self.offset)
    }
}

/// Selección activa de texto, posiblemente a través de varios bloques.
#[derive(Clone, Debug)]
struct Selection {
    chapter: usize,
    /// Extremo fijo: donde empezó el arrastre o el clic con Mayúsculas.
    anchor: Anchor,
    /// Extremo móvil: donde está el cursor o el puntero.
    focus: Anchor,
    quote: String,
    /// Texto seleccionado por bloque, en orden.
    parts: Vec<String>,
    /// Rect en pantalla, para ubicar la barra flotante.
    rect: egui::Rect,
    /// Id del `TextEdit` que la contiene; se usa para soltar el foco.
    edit_id: egui::Id,
}

impl Selection {
    fn ordered(&self) -> (Anchor, Anchor) {
        if self.anchor.key() <= self.focus.key() {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }

    fn start_block(&self) -> usize {
        self.ordered().0.block
    }

    fn start_offset(&self) -> usize {
        self.ordered().0.offset
    }

    fn end_block(&self) -> usize {
        self.ordered().1.block
    }

    fn end_offset(&self) -> usize {
        self.ordered().1.offset
    }

    fn is_multi_block(&self) -> bool {
        self.start_block() != self.end_block()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Segment {
    block: usize,
    start: usize,
    end: usize,
}

/// Enlace interno clickeado, con la posición para anclar la vista previa.
#[derive(Clone, Debug)]
struct LinkClick {
    spine: usize,
    anchor: Option<String>,
    rect: egui::Rect,
    label: String,
}

/// Vista previa de una nota o enlace interno. No navega hasta confirmar.
#[derive(Clone, Debug)]
struct LinkPreview {
    spine: usize,
    anchor: Option<String>,
    chapter_title: String,
    text: String,
    rect: egui::Rect,
    is_note: bool,
}

/// Geometría de un bloque de texto dibujado en el frame actual. Sirve para
/// mapear el puntero a un bloque y offset cuando la selección sale del
/// `TextEdit` que tiene el foco.
struct BlockGeom {
    block: usize,
    rect: egui::Rect,
    galley: std::sync::Arc<egui::Galley>,
    prefix_len: usize,
    text_len: usize,
}

/// Construye una selección normalizada a partir de dos extremos. Devuelve
/// `None` si no cubre texto visible.
fn selection_from_anchors(
    chapter: usize,
    blocks: &[Block],
    anchor: Anchor,
    focus: Anchor,
    rect: egui::Rect,
    edit_id: egui::Id,
) -> Option<Selection> {
    if blocks.is_empty() {
        return None;
    }
    let clamp = |anchor: Anchor| {
        let block = anchor.block.min(blocks.len() - 1);
        Anchor {
            block,
            offset: anchor.offset.min(blocks[block].text().chars().count()),
        }
    };
    let anchor = clamp(anchor);
    let focus = clamp(focus);
    let (start, end) = if anchor.key() <= focus.key() {
        (anchor, focus)
    } else {
        (focus, anchor)
    };
    let mut parts = Vec::new();
    for (block, current) in blocks
        .iter()
        .enumerate()
        .take(end.block + 1)
        .skip(start.block)
    {
        let text = current.text();
        if text.trim().is_empty() {
            continue;
        }
        let len = text.chars().count();
        let from = if block == start.block {
            start.offset.min(len)
        } else {
            0
        };
        let to = if block == end.block {
            end.offset.min(len)
        } else {
            len
        };
        if to > from {
            parts.push(text.chars().skip(from).take(to - from).collect::<String>());
        }
    }
    if parts.iter().all(|part| part.trim().is_empty()) {
        return None;
    }
    let quote = parts.join("\n");
    Some(Selection {
        chapter,
        anchor,
        focus,
        quote,
        parts,
        rect,
        edit_id,
    })
}

/// Convierte una selección activa en la anotación que se va a persistir.
fn annotation_from_selection(
    selection: &Selection,
    book: String,
    note: String,
    color: u8,
) -> Annotation {
    Annotation {
        book,
        chapter: selection.chapter,
        block: selection.start_block(),
        start: selection.start_offset(),
        end: selection.end_offset(),
        end_block: selection.is_multi_block().then(|| selection.end_block()),
        parts: selection.parts.clone(),
        quote: selection.quote.clone(),
        note,
        color,
        ..Annotation::default()
    }
}

/// Segmentos por bloque de una selección activa, para previsualizar el rango
/// mientras se arrastra.
fn selection_segments(selection: &Selection, blocks: &[Block]) -> Vec<Segment> {
    let (start, end) = selection.ordered();
    let mut out = Vec::new();
    if start.block >= blocks.len() {
        return out;
    }
    let last = end.block.min(blocks.len() - 1);
    for (block, current) in blocks.iter().enumerate().take(last + 1).skip(start.block) {
        let len = current.text().chars().count();
        let from = if block == start.block {
            start.offset.min(len)
        } else {
            0
        };
        let to = if block == end.block {
            end.offset.min(len)
        } else {
            len
        };
        if to > from {
            out.push(Segment {
                block,
                start: from,
                end: to,
            });
        }
    }
    out
}

/// Bloque y offset más cercanos a un punto de pantalla. `upward` sesga la
/// elección al bloque anterior cuando el puntero está en el espacio entre dos
/// párrafos.
fn pointer_anchor(geoms: &[BlockGeom], pos: egui::Pos2, upward: bool) -> Option<Anchor> {
    if geoms.is_empty() {
        return None;
    }
    let geom = if upward {
        geoms
            .iter()
            .rev()
            .find(|geom| geom.rect.top() < pos.y)
            .or_else(|| geoms.first())
    } else {
        geoms
            .iter()
            .find(|geom| geom.rect.bottom() > pos.y)
            .or_else(|| geoms.last())
    }?;
    let index = geom
        .galley
        .cursor_from_pos(pos - geom.rect.min)
        .index
        .saturating_sub(geom.prefix_len);
    Some(Anchor {
        block: geom.block,
        offset: index.min(geom.text_len),
    })
}

/// Rect de pantalla de un extremo, para ubicar la barra flotante.
fn anchor_screen_rect(geoms: &[BlockGeom], anchor: Anchor) -> Option<egui::Rect> {
    let geom = geoms.iter().find(|geom| geom.block == anchor.block)?;
    let cursor = egui::text::CCursor::new(
        anchor
            .offset
            .saturating_add(geom.prefix_len)
            .min(geom.text_len + geom.prefix_len),
    );
    Some(
        geom.galley
            .pos_from_cursor(cursor)
            .translate(geom.rect.min.to_vec2()),
    )
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

struct EpubEditor {
    index: usize,
    path: PathBuf,
    title: String,
    author: String,
    lang: String,
    cover_ext: CoverExt,
    cover_upload: Option<Vec<u8>>,
    cover_texture: Option<egui::TextureHandle>,
    error: Option<String>,
}

struct AnnRow {
    annotation: Annotation,
    book_title: String,
    chapter_title: String,
    missing: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigTab {
    Lectura,
    Biblioteca,
    Anotaciones,
    Consultas,
    Privacidad,
    Avanzado,
}

/// Estado de la comprobación/descarga de una release.
#[derive(Clone, Debug)]
enum UpdateState {
    Idle,
    Checking,
    UpToDate,
    Available {
        release: update::ReleaseInfo,
        error: Option<String>,
    },
    Downloading(update::ReleaseInfo),
    Downloaded {
        release: update::ReleaseInfo,
        path: PathBuf,
    },
    Error(String),
}

#[derive(Debug)]
enum UpdateMsg {
    Checked(Result<Option<update::ReleaseInfo>, String>),
    Downloaded {
        release: update::ReleaseInfo,
        result: Result<PathBuf, String>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConsultaTab {
    Diccionario,
    Traduccion,
    Ia,
    Historial,
}

#[derive(Clone)]
struct ConsultaTarget {
    text: String,
    context: String,
    source_lang: String,
    book: String,
    title: String,
    author: String,
    chapter: usize,
    block: usize,
}

const CARD_W: f32 = 160.0;
const CARD_COVER_H: f32 = 240.0;
const CARD_GAP: f32 = 20.0;
const PAGE_PAD: f32 = 24.0;
const CARD_TEXT_H: f32 = 36.0;
const CARD_AUTHOR_H: f32 = 34.0;
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
    scan_rx: Option<Receiver<(Library, ScanStats)>>,
    scanning: bool,
    confirm_remove_book: Option<usize>,
    confirm_remove_folder: Option<String>,
    confirm_recover_book: Option<usize>,
    library_search: String,
    library_order: LibraryOrder,
    library_filter: LibraryFilter,
    library_content: Option<LibraryContentSearch>,
    cfg: Config,
    ai_key: secrets::KeyState,
    ai_key_input: String,
    confirm_delete_ai_key: bool,
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
    link_preview: Option<LinkPreview>,
    external_link: Option<String>,
    back_spot: Option<Progress>,
    update_state: UpdateState,
    update_rx: Option<Receiver<UpdateMsg>>,
    data_notice: Option<String>,
    note: Option<NoteEditor>,
    epub_editor: Option<EpubEditor>,
    show_annotations: bool,
    annotations_global: bool,
    notes_query: String,
    confirm_delete_annotation: Option<u64>,
    show_bookmarks: bool,
    active_color: u8,
    consulta_history: ConsultaHistory,
    consulta_session: String,
    show_consulta: bool,
    consulta_tab: ConsultaTab,
    consulta_target: Option<ConsultaTarget>,
    consulta_result: Option<ConsultaResponse>,
    consulta_error: Option<String>,
    consulta_busy: bool,
    consulta_rx: Option<Receiver<(u64, Result<ConsultaResponse, String>)>>,
    consulta_id: u64,
    consulta_prompt: AiPrompt,
    consulta_pending_request: Option<ConsultaRequest>,
    consulta_pending_prompt: Option<AiPrompt>,
    consulta_history_view: Option<ConsultaEntry>,
    consulta_confirm_clear: bool,
    show_config: bool,
    config_tab: ConfigTab,
    config_confirm_clear: bool,
    config_confirm_reset: bool,
    show_help: bool,
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
    was_focused: bool,
    last_auto_scan: Option<Instant>,
    dev: DevOpts,
    shot_frames: u32,
    shot_requested: bool,
    window_title: String,
}

/// Banderas de desarrollo: captura e inspección visual sin tocar la UI.
#[derive(Debug, Clone, Default)]
pub struct DevOpts {
    pub shot: Option<PathBuf>,
    /// Tamaño de ventana para capturas responsivas (`WxH`).
    pub size: Option<(f32, f32)>,
    /// Abre el diálogo de recuperación del primer EPUB roto/faltante.
    pub recover: bool,
    /// Abre la confirmación de quitar el primer libro.
    pub confirm_remove: bool,
    /// Abre la confirmación de dejar de vigilar la primera carpeta.
    pub confirm_folder: bool,
    /// Dispara un reescaneo al abrir (capturas del estado busy).
    pub rescan: bool,
    /// Pone foco de teclado en la primera tarjeta (capturas de foco).
    pub focus_card: bool,
    /// Abre la confirmación de borrar el primer destacado (capturas).
    pub confirm_delete: bool,
    /// Muestra la barra de selección con una selección sintética (capturas).
    pub selection: bool,
    /// Abre una vista previa de enlace interno (capturas).
    pub link_preview: bool,
    /// Inyecta una release simulada en Configuración (capturas).
    pub update_demo: bool,
    /// Abre la confirmación de enlace externo (capturas).
    pub external_link_demo: bool,
    /// Abre la ventana de atajos (capturas).
    pub help: bool,
    /// Abre el editor de metadatos/portada del primer libro (capturas).
    pub edit: bool,
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
    /// Texto sintético para abrir el panel de consulta (capturas).
    pub consulta: Option<String>,
    /// `diccionario`, `traduccion` o `ia` para la consulta sintética.
    pub consulta_kind: Option<String>,
    /// Inyecta un resultado falso en el panel (capturas, sin red).
    pub consulta_demo: bool,
    /// Ejecuta la consulta sintética al abrir (verificación real).
    pub consulta_run: bool,
    /// Exporta las anotaciones a esta ruta sin abrir el diálogo (dev/capturas).
    pub export_md: Option<PathBuf>,
    /// Salta a un ancla del libro al abrir (capturas/dev).
    pub anchor: Option<String>,
    /// `recent`, `title`, `author`, `series` o `added` (capturas/dev).
    pub library_order: Option<String>,
    /// `all`, `reading`, `unread`, `missing` o `duplicates` (capturas/dev).
    pub library_filter: Option<String>,
    /// Filtra la biblioteca por nombre de serie (capturas/dev).
    pub library_series: Option<String>,
    /// Abre el panel de marcadores al iniciar el lector (capturas).
    pub bookmarks: bool,
    /// Desplaza la grilla de biblioteca a este offset (capturas de virtualización).
    pub library_scroll: Option<f32>,
    /// Ejecuta una búsqueda full-text en la biblioteca al iniciar (capturas).
    pub library_content_query: Option<String>,
}

impl Lector {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>, dev: DevOpts) -> Self {
        fonts::install(&cc.egui_ctx);
        let (mut cfg, cfg_warning) = Config::load_with_report();
        let (annotations, ann_warning) = Annotations::load();
        let (consulta_history, history_warning) = ConsultaHistory::load();
        let (mut library, library_warning) = if let Some(path) = dev.library_path.as_ref() {
            Library::load_with_report_from(path)
        } else {
            Library::load_with_report()
        };
        let migrated = library.migrate_progress_from(&mut cfg);
        let mut warning = cfg_warning
            .or(library_warning)
            .or(ann_warning)
            .or(history_warning);
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
        #[cfg(target_os = "macos")]
        crate::macos::set_app_icon();

        let theme_idx = cfg.settings.theme;
        let ai_key = secrets::load(&cfg.consulta.ai_provider);
        let initial_delete = if dev.confirm_delete {
            annotations.sorted_all().first().map(|a| a.id)
        } else {
            None
        };
        let initial_recover = if dev.recover {
            library
                .books
                .iter()
                .position(|b| b.missing || b.error.is_some())
        } else {
            None
        };
        let initial_remove = if dev.confirm_remove && !library.books.is_empty() {
            Some(0)
        } else {
            None
        };
        let initial_folder = if dev.confirm_folder {
            library.folders.first().cloned()
        } else {
            None
        };
        let initial_search = dev.search.clone().unwrap_or_default();
        let initial_book_search = dev.book_search.clone().unwrap_or_default();
        let update_state = if dev.update_demo {
            UpdateState::Available {
                release: update::demo_release(),
                error: None,
            }
        } else {
            UpdateState::Idle
        };
        let mut app = Self {
            screen: Screen::Library,
            book: None,
            chapter: 0,
            blocks: Vec::new(),
            textures: HashMap::new(),
            library,
            library_textures: HashMap::new(),
            scan_rx: None,
            scanning: false,
            confirm_remove_book: initial_remove,
            confirm_remove_folder: initial_folder,
            confirm_recover_book: initial_recover,
            confirm_delete_annotation: initial_delete,
            library_search: initial_search,
            library_order: LibraryOrder::Recent,
            library_filter: LibraryFilter::All,
            library_content: None,
            cfg,
            ai_key,
            ai_key_input: String::new(),
            confirm_delete_ai_key: false,
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
            link_preview: None,
            external_link: None,
            back_spot: None,
            update_state,
            update_rx: None,
            data_notice: None,
            note: None,
            epub_editor: None,
            show_annotations: false,
            annotations_global: false,
            notes_query: String::new(),
            show_bookmarks: false,
            active_color: 0,
            consulta_history,
            consulta_session: new_session_id(),
            show_consulta: false,
            consulta_tab: ConsultaTab::Diccionario,
            consulta_target: None,
            consulta_result: None,
            consulta_error: None,
            consulta_busy: false,
            consulta_rx: None,
            consulta_id: 0,
            consulta_prompt: AiPrompt::Explicar,
            consulta_pending_request: None,
            consulta_pending_prompt: None,
            consulta_history_view: None,
            consulta_confirm_clear: false,
            show_config: false,
            config_tab: ConfigTab::Consultas,
            config_confirm_clear: false,
            config_confirm_reset: false,
            show_help: dev.help,
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
            was_focused: true,
            last_auto_scan: None,
            dev,
            shot_frames: 0,
            shot_requested: false,
            window_title: "Candil".into(),
        };
        app.applied_theme = Some(app.cfg.settings.theme);

        let force_library = app.dev.screen.as_deref() == Some("library");
        let target = if force_library {
            None
        } else {
            path.or_else(|| {
                if app.dev.resume || app.cfg.startup == config::StartupMode::LastBook {
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
        if app.screen == Screen::Library && app.dev.panel.as_deref() == Some("config") {
            app.show_config = true;
            app.config_tab = ConfigTab::Biblioteca;
        }
        if app.screen == Screen::Library && app.dev.panel.as_deref() == Some("advanced") {
            app.show_config = true;
            app.config_tab = ConfigTab::Avanzado;
        }
        if app.dev.update_demo {
            app.show_config = true;
            app.config_tab = ConfigTab::Avanzado;
        }
        if app.dev.external_link_demo {
            app.external_link = Some("https://example.com/candil".to_string());
        }
        if app.screen == Screen::Library && app.dev.edit {
            if let Some(index) = app
                .library
                .books
                .iter()
                .position(|book| !book.missing && book.error.is_none())
            {
                app.open_epub_editor(index, &cc.egui_ctx);
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
            if app.dev.selection {
                app.seed_dev_selection();
            }
            if app.dev.link_preview {
                app.seed_dev_link_preview();
            }
        }
        if let Some(order) = app.dev.library_order.as_deref() {
            app.library_order = match order {
                "title" => LibraryOrder::Title,
                "author" => LibraryOrder::Author,
                "series" => LibraryOrder::Series,
                "added" => LibraryOrder::Added,
                _ => LibraryOrder::Recent,
            };
        }
        if let Some(series) = app.dev.library_series.clone() {
            app.library_filter = LibraryFilter::Series(series);
        } else if let Some(filter) = app.dev.library_filter.as_deref() {
            app.library_filter = match filter {
                "reading" => LibraryFilter::Reading,
                "unread" => LibraryFilter::Unread,
                "missing" => LibraryFilter::Missing,
                "duplicates" => LibraryFilter::Duplicates,
                _ => LibraryFilter::All,
            };
        }
        if app.dev.bookmarks && app.book.is_some() {
            app.show_bookmarks = true;
            app.show_toc = false;
            app.show_settings = false;
            app.show_annotations = false;
            app.show_search = false;
            app.show_goto = false;
        }
        if let Some(query) = app.dev.library_content_query.clone() {
            if app.screen == Screen::Library {
                app.library_search = query;
                app.start_library_content_search(&cc.egui_ctx);
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
                self.show_consulta = false;
                self.show_config = false;
                self.consulta_target = None;
                self.consulta_result = None;
                self.consulta_error = None;
                self.consulta_rx = None;
                self.consulta_busy = false;
                self.consulta_pending_request = None;
                self.consulta_pending_prompt = None;
                self.consulta_history_view = None;
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
                    Some("config") => {
                        self.show_config = true;
                        self.config_tab = ConfigTab::Consultas;
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
                if let Some(text) = self.dev.consulta.clone() {
                    let (source_lang, book_path, title, author) = self
                        .book
                        .as_ref()
                        .map(|book| {
                            (
                                consulta::normalize_lang(&book.lang),
                                book.path.to_string_lossy().to_string(),
                                book.title.clone(),
                                book.author.clone(),
                            )
                        })
                        .unwrap_or_else(|| {
                            ("en".into(), String::new(), String::new(), String::new())
                        });
                    let context = self
                        .blocks
                        .first()
                        .map(|block| block.text())
                        .unwrap_or_default();
                    self.consulta_target = Some(ConsultaTarget {
                        text,
                        context,
                        source_lang,
                        book: book_path,
                        title,
                        author,
                        chapter: self.chapter,
                        block: 0,
                    });
                    self.consulta_tab = match self.dev.consulta_kind.as_deref() {
                        Some("traduccion") | Some("translation") => ConsultaTab::Traduccion,
                        Some("ia") | Some("ai") => ConsultaTab::Ia,
                        Some("historial") | Some("history") => ConsultaTab::Historial,
                        _ => ConsultaTab::Diccionario,
                    };
                    self.show_consulta = true;
                    self.config_tab = ConfigTab::Consultas;
                    if self.dev.consulta_demo {
                        if self.consulta_tab == ConsultaTab::Historial {
                            self.consulta_history_view = self
                                .consulta_history
                                .newest_first()
                                .first()
                                .map(|entry| (*entry).clone());
                        } else {
                            let kind = match self.consulta_tab {
                                ConsultaTab::Traduccion => ConsultaKind::Traduccion,
                                ConsultaTab::Ia => ConsultaKind::Ia,
                                _ => ConsultaKind::Diccionario,
                            };
                            self.consulta_result = Some(demo_response(kind));
                        }
                    } else if self.dev.consulta_run {
                        let kind = match self.consulta_tab {
                            ConsultaTab::Traduccion => ConsultaKind::Traduccion,
                            ConsultaTab::Ia => ConsultaKind::Ia,
                            _ => ConsultaKind::Diccionario,
                        };
                        self.run_consulta(kind);
                    }
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
        self.link_preview = None;
        self.back_spot = None;
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
        self.back_spot = None;
    }

    /// Abre la vista previa de un enlace interno sin cambiar de posición.
    fn open_link_preview(&mut self, click: LinkClick) {
        let is_note = looks_like_note_marker(&click.label);
        let (text, chapter_title) = if click.spine == self.chapter {
            let text = preview_text(&self.blocks, click.anchor.as_deref(), 600);
            let title = self
                .book
                .as_mut()
                .map(|book| book.chapter_title(click.spine))
                .unwrap_or_default();
            (text, title)
        } else {
            let blocks = self
                .book
                .as_mut()
                .map(|book| book.chapter_blocks(click.spine))
                .unwrap_or_default();
            let text = preview_text(&blocks, click.anchor.as_deref(), 600);
            let title = self
                .book
                .as_mut()
                .map(|book| book.chapter_title(click.spine))
                .unwrap_or_default();
            (text, title)
        };
        self.link_preview = Some(LinkPreview {
            spine: click.spine,
            anchor: click.anchor,
            chapter_title,
            text,
            rect: click.rect,
            is_note,
        });
        self.selection = None;
        self.reader_focus = None;
    }

    /// Salta al destino de la vista previa y guarda el punto de regreso.
    fn jump_from_preview(&mut self, preview: &LinkPreview) {
        let spot = self.current_progress();
        self.link_preview = None;
        self.jump_to(preview.spine, preview.anchor.clone());
        self.back_spot = spot;
    }

    /// Restaura el capítulo, bloque y fracción anteriores.
    fn go_back(&mut self, ctx: &egui::Context) {
        let Some(spot) = self.back_spot.take() else {
            return;
        };
        self.link_preview = None;
        self.selection = None;
        self.note = None;
        if spot.chapter != self.chapter {
            self.load_chapter(spot.chapter, Some(spot));
        } else {
            self.pending_restore = Some(spot);
        }
        ctx.request_repaint();
    }

    fn link_preview_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(preview) = self.link_preview.clone() else {
            return;
        };
        let screen = ctx.screen_rect();
        let width = 380.0f32.min((screen.width() - 24.0).max(240.0));
        let estimated_height = 200.0;
        let mut pos = egui::pos2(preview.rect.left(), preview.rect.bottom() + 10.0);
        if pos.y + estimated_height > screen.bottom() - 8.0 {
            pos.y = (preview.rect.top() - estimated_height - 10.0).max(screen.top() + 8.0);
        }
        let max_x = (screen.right() - width - 12.0).max(screen.left() + 12.0);
        pos.x = pos.x.clamp(screen.left() + 12.0, max_x);

        let mut jump = false;
        let mut close = false;
        egui::Area::new(egui::Id::new("link-preview"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(egui::Stroke::new(1.0, th.border))
                    .corner_radius(theme::radius::MD)
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.set_width(width - 24.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(if preview.is_note {
                                    "Nota al pie"
                                } else {
                                    "Enlace interno"
                                })
                                .size(theme::text::LABEL)
                                .strong()
                                .color(th.fg),
                            );
                            if !preview.chapter_title.is_empty() {
                                ui.label(
                                    RichText::new(format!(
                                        "· {}",
                                        truncate(&preview.chapter_title, 34)
                                    ))
                                    .size(theme::text::META)
                                    .color(th.dim),
                                );
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                                    .on_hover_text("Volver")
                                    .clicked()
                                {
                                    close = true;
                                }
                            });
                        });
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(180.0)
                            .show(ui, |ui| {
                                let empty = preview.text.is_empty();
                                let text = if empty {
                                    "El destino no tiene texto legible."
                                } else {
                                    preview.text.as_str()
                                };
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(text)
                                            .size(theme::text::BODY)
                                            .color(if empty { th.dim } else { th.fg }),
                                    )
                                    .wrap(),
                                );
                            });
                        ui.add_space(theme::space::SM);
                        ui.horizontal(|ui| {
                            if widgets::button(ui, th, "Ir al texto", widgets::ButtonKind::Primary)
                                .clicked()
                            {
                                jump = true;
                            }
                            if widgets::button(ui, th, "Volver", widgets::ButtonKind::Secondary)
                                .clicked()
                            {
                                close = true;
                            }
                        });
                    });
            });

        if jump {
            self.jump_from_preview(&preview);
            ctx.request_repaint();
        } else if close {
            self.link_preview = None;
        }
    }

    fn external_link_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(url) = self.external_link.clone() else {
            return;
        };
        let mut open = false;
        let mut cancel = false;
        let _ = egui::Modal::new(egui::Id::new("external-link-modal"))
            .backdrop_color(Color32::from_black_alpha(40))
            .frame(widgets::panel_frame(th))
            .show(ctx, |ui| {
                ui.set_width(460.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Abrir enlace externo")
                            .size(theme::text::TITLE)
                            .strong()
                            .color(th.fg),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                            .on_hover_text("Cancelar")
                            .clicked()
                        {
                            cancel = true;
                        }
                    });
                });
                ui.separator();
                ui.add(
                    egui::Label::new(
                        RichText::new(truncate(&url, 180))
                            .size(theme::text::META)
                            .color(th.dim),
                    )
                    .wrap(),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new("Se abrirá con la aplicación del sistema.")
                            .size(theme::text::META)
                            .color(th.dim),
                    )
                    .wrap(),
                );
                ui.add_space(theme::space::SM);
                ui.horizontal(|ui| {
                    if widgets::button(ui, th, "Abrir enlace", widgets::ButtonKind::Primary)
                        .clicked()
                    {
                        open = true;
                    }
                    if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if open {
            open_external(&url);
            self.external_link = None;
        } else if cancel {
            self.external_link = None;
        }
    }

    fn back_to_text(&mut self, ctx: &egui::Context, th: &Theme) {
        if self.back_spot.is_none() {
            return;
        }
        let mut back = false;
        egui::Area::new(egui::Id::new("back-to-text"))
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(24.0, -30.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                if widgets::button(ui, th, "← Volver al texto", widgets::ButtonKind::Secondary)
                    .on_hover_text("Volver al punto anterior")
                    .clicked()
                {
                    back = true;
                }
            });
        if back {
            self.go_back(ctx);
        }
    }

    /// Capturas: abre la vista previa del primer enlace interno del capítulo.
    fn seed_dev_link_preview(&mut self) {
        let Some(book) = self.book.as_ref() else {
            return;
        };
        let Some((href, label)) = self.blocks.iter().find_map(|block| {
            block.runs.iter().find_map(|run| {
                run.href
                    .as_ref()
                    .map(|href| (href.clone(), run.text.trim().to_string()))
            })
        }) else {
            return;
        };
        let Some((spine, anchor)) = book.link_target(&href, self.chapter) else {
            return;
        };
        self.open_link_preview(LinkClick {
            spine,
            anchor,
            rect: egui::Rect::from_min_size(egui::pos2(340.0, 260.0), egui::vec2(1.0, 1.0)),
            label,
        });
    }

    /// Capturas: siembra una selección de dos bloques en el capítulo actual.
    /// Si hay un ancla de desarrollo, empieza por su bloque.
    fn seed_dev_selection(&mut self) {
        let start = self
            .dev
            .anchor
            .as_ref()
            .and_then(|anchor| {
                self.blocks
                    .iter()
                    .position(|block| block.anchor.as_deref() == Some(anchor.as_str()))
            })
            .unwrap_or(0);
        let candidates: Vec<(usize, usize)> = self
            .blocks
            .iter()
            .enumerate()
            .skip(start)
            .filter_map(|(index, block)| {
                let len = block.text().chars().count();
                (!matches!(block.kind, Kind::Heading(_)) && len >= 10).then_some((index, len))
            })
            .take(2)
            .collect();
        let Some(&(first, first_len)) = candidates.first() else {
            return;
        };
        let anchor = Anchor {
            block: first,
            offset: 0,
        };
        let focus = candidates.get(1).map_or(
            Anchor {
                block: first,
                offset: first_len.min(24),
            },
            |&(second, second_len)| Anchor {
                block: second,
                offset: second_len.min(24),
            },
        );
        self.selection = selection_from_anchors(
            self.chapter,
            &self.blocks,
            anchor,
            focus,
            egui::Rect::from_min_size(egui::pos2(320.0, 260.0), egui::vec2(300.0, 40.0)),
            egui::Id::new("dev-selection"),
        );
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

    /// Posición actual como `Progress`, con la misma lógica que la persistencia.
    fn current_progress(&self) -> Option<Progress> {
        self.book.as_ref()?;
        let ratio = if self.max_scroll > 1.0 {
            (self.scroll / self.max_scroll).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let anchor = top_block(self.scroll, &self.block_positions);
        let anchor_ratio = anchor.map(|index| {
            block_fraction(self.scroll, index, &self.block_positions, self.max_scroll)
        });
        Some(Progress {
            chapter: self.chapter,
            ratio,
            anchor,
            anchor_ratio,
            legacy_offset: None,
        })
    }

    fn bookmark_matches(a: &Progress, b: &Progress) -> bool {
        if a.chapter != b.chapter {
            return false;
        }
        match (a.anchor, b.anchor) {
            (Some(a_anchor), Some(b_anchor)) => {
                a_anchor == b_anchor
                    && (a.anchor_ratio.unwrap_or(0.0) - b.anchor_ratio.unwrap_or(0.0)).abs() < 0.01
            }
            (None, None) => (a.ratio - b.ratio).abs() < 0.01,
            _ => false,
        }
    }

    fn add_bookmark_here(&mut self) {
        let Some(progress) = self.current_progress() else {
            return;
        };
        let Some(key) = self.book_key() else {
            return;
        };
        let Some(entry) = self.library.books.iter_mut().find(|b| b.path == key) else {
            return;
        };
        if entry
            .bookmarks
            .iter()
            .any(|bookmark| Self::bookmark_matches(&bookmark.progress, &progress))
        {
            self.say("Ya hay un marcador en esta posición.");
            return;
        }
        entry.add_bookmark(progress, String::new());
        self.save_library();
        self.say("Marcador añadido.");
    }

    fn remove_bookmark(&mut self, id: u64) {
        let Some(key) = self.book_key() else {
            return;
        };
        let Some(entry) = self.library.books.iter_mut().find(|b| b.path == key) else {
            return;
        };
        if entry.remove_bookmark(id) {
            self.save_library();
            self.say("Marcador eliminado.");
        }
    }

    fn jump_to_bookmark(&mut self, progress: Progress) {
        if self.book.is_none() {
            return;
        }
        self.show_bookmarks = false;
        self.show_annotations = false;
        self.show_goto = false;
        self.show_search = false;
        self.selection = None;
        self.note = None;
        self.reader_focus = None;
        self.pending_anchor = None;
        self.pending_block_jump = None;
        self.pending_search_jump = None;
        self.search_highlight = None;
        self.load_chapter(progress.chapter, Some(progress));
        self.dirty = true;
    }

    fn persist(&mut self) {
        let key = self
            .book
            .as_ref()
            .map(|book| book.path.to_string_lossy().to_string());
        if let (Some(key), Some(progress)) = (key, self.current_progress()) {
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

    fn relink_book(&mut self, index: usize, path: PathBuf) {
        let old_path = self.library.books.get(index).map(|b| b.path.clone());
        match self.library.relink(index, &path) {
            Ok(new_path) => {
                if let Some(old_path) = old_path {
                    let changed = self.annotations.rekey(&old_path, &new_path);
                    if changed > 0 {
                        self.save_annotations();
                    }
                    if let Some(recent) =
                        self.cfg.recent.iter_mut().find(|p| p.as_str() == old_path)
                    {
                        *recent = new_path.clone();
                        if let Err(e) = self.cfg.save() {
                            self.say(format!("No pude guardar la configuración: {e}"));
                        }
                    }
                }
                self.library_textures.clear();
                self.save_library();
                self.say("Libro reubicado en la biblioteca.");
            }
            Err(e) => self.say(e),
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
        self.show_bookmarks = false;
        self.show_consulta = false;
        self.show_config = false;
        self.consulta_target = None;
        self.consulta_result = None;
        self.consulta_error = None;
        self.consulta_rx = None;
        self.consulta_busy = false;
        self.consulta_pending_request = None;
        self.consulta_pending_prompt = None;
        self.consulta_history_view = None;
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
        self.annotations.add(annotation_from_selection(
            &selection,
            book,
            String::new(),
            self.active_color.min(2),
        ));
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
            self.annotations.add(annotation_from_selection(
                &selection,
                editor.book,
                draft,
                editor.color.min(2),
            ));
        }
        self.save_annotations();
        self.say("Nota guardada.");
    }

    fn remove_annotation(&mut self, id: u64) {
        if self.annotations.remove(id) {
            self.save_annotations();
            self.say("Destacado borrado.");
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
        self.pending_block_jump = Some((annotation.block, annotation.first_part()));
        self.show_annotations = false;
        self.selection = None;
        self.note = None;
        self.reader_focus = None;
        self.back_spot = None;
    }

    fn open_consulta(&mut self, ctx: &egui::Context) {
        let Some(selection) = self.selection.clone() else {
            return;
        };
        let Some(book) = self.book.as_ref() else {
            return;
        };
        let text = selection
            .quote
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if text.is_empty() {
            return;
        }
        // Si había una consulta en vuelo, se descarta: el texto cambió.
        self.consulta_rx = None;
        self.consulta_busy = false;
        self.consulta_pending_request = None;
        self.consulta_pending_prompt = None;
        self.consulta_id = self.consulta_id.wrapping_add(1);
        let start_block = selection.start_block();
        let start_offset = selection.start_offset();
        let block_text = self
            .blocks
            .get(start_block)
            .map(|block| block.text())
            .unwrap_or_default();
        let end_offset = if selection.is_multi_block() {
            block_text.chars().count()
        } else {
            selection.end_offset()
        };
        let context = context_around(&block_text, start_offset, end_offset, 1_500);
        let source_lang = consulta::normalize_lang(&book.lang);
        let book_path = book.path.to_string_lossy().to_string();
        let title = book.title.clone();
        let author = book.author.clone();
        self.consulta_target = Some(ConsultaTarget {
            text: text.clone(),
            context,
            source_lang,
            book: book_path,
            title,
            author,
            chapter: selection.chapter,
            block: start_block,
        });
        self.consulta_tab = if text.split_whitespace().count() <= 1 {
            ConsultaTab::Diccionario
        } else {
            ConsultaTab::Traduccion
        };
        self.consulta_result = None;
        self.consulta_error = None;
        self.consulta_history_view = None;
        self.consulta_confirm_clear = false;
        self.show_consulta = true;
        self.show_toc = false;
        self.show_settings = false;
        self.show_annotations = false;
        self.show_bookmarks = false;
        self.show_search = false;
        self.show_goto = false;
        self.show_config = false;
        self.surrender_reader_focus(ctx);
        ctx.request_repaint();
    }

    fn open_config(&mut self, ctx: &egui::Context, tab: ConfigTab) {
        self.show_config = true;
        self.config_tab = tab;
        self.show_toc = false;
        self.show_settings = false;
        self.show_search = false;
        self.show_goto = false;
        self.show_annotations = false;
        self.show_bookmarks = false;
        self.show_consulta = false;
        self.consulta_rx = None;
        self.consulta_busy = false;
        self.consulta_pending_request = None;
        self.consulta_pending_prompt = None;
        self.selection = None;
        self.note = None;
        ctx.memory_mut(|memory| memory.surrender_focus(egui::Id::new("note-editor")));
        self.config_confirm_clear = false;
        self.config_confirm_reset = false;
        self.surrender_reader_focus(ctx);
        ctx.request_repaint();
    }

    fn help_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_help {
            return;
        }
        let mut open = self.show_help;
        let help_h = (ctx.screen_rect().height() - 180.0).clamp(260.0, 560.0);
        egui::Window::new(
            RichText::new("Atajos")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_MD)
        .max_height(help_h + 90.0)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            widgets::meta(ui, th, "Esc cierra esta ventana");
            let sections: [(&str, &[(&str, &str)]); 4] = [
                (
                    "Lectura",
                    &[
                        ("Space / PgDn / ↓", "Avanzar"),
                        ("PgUp / ↑", "Retroceder"),
                        ("Home", "Inicio del capítulo"),
                        ("+ / −", "Tamaño de fuente"),
                    ],
                ),
                (
                    "Navegación",
                    &[
                        ("← / →", "Capítulo anterior / siguiente"),
                        ("T", "Índice"),
                        ("G", "Ir a porcentaje o capítulo"),
                        ("⌘F", "Buscar en el libro"),
                        ("⌘O", "Abrir un EPUB"),
                        ("Esc", "Cerrar panel o volver"),
                    ],
                ),
                (
                    "Anotaciones",
                    &[
                        ("H", "Destacar selección"),
                        ("N", "Nota sobre la selección"),
                        ("D", "Destacados y notas"),
                        ("K", "Consultar selección"),
                        ("Mayús+K", "Traducir selección"),
                    ],
                ),
                (
                    "Paneles",
                    &[
                        ("B", "Guardar marcador"),
                        ("Mayús+B", "Marcadores"),
                        ("S", "Tipografía"),
                        ("⌘,", "Configuración"),
                        ("F1 / ?", "Esta ayuda"),
                    ],
                ),
            ];
            for (title, rows) in sections {
                widgets::section_header(ui, th, title);
                egui::Grid::new(format!("help-{title}"))
                    .num_columns(2)
                    .spacing([16.0, 4.0])
                    .show(ui, |ui| {
                        for (keys, action) in rows {
                            ui.label(
                                RichText::new(*keys)
                                    .size(theme::text::META)
                                    .strong()
                                    .color(th.fg),
                            );
                            ui.label(RichText::new(*action).size(theme::text::META).color(th.dim));
                            ui.end_row();
                        }
                    });
                ui.add_space(theme::space::XS);
            }
        });
        self.show_help = open;
    }

    fn config_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_config {
            return;
        }
        let folders = self.library.folders.clone();
        let book_count = self.library.books.len();
        let missing = self.library.books.iter().filter(|b| b.missing).count();
        let annotation_count = self.annotations.items.len();
        let history_count = self.consulta_history.items.len();
        let was_open = self.show_config;
        let mut open = self.show_config;
        let mut add_folder = false;
        let mut rescan = false;
        let mut remove_folder: Option<String> = None;
        let mut open_typography = false;
        let mut open_annotations = false;
        let mut export_all = false;
        let mut open_data = false;
        let mut reset_config = false;
        let mut clear_history = false;
        let mut changed = false;
        let mut save_ai_key = false;
        let mut delete_ai_key = false;
        let mut check_update = false;
        let mut download_update: Option<update::ReleaseInfo> = None;
        let mut open_release: Option<String> = None;
        let mut reveal_download: Option<PathBuf> = None;
        let mut export_data = false;
        let mut import_data = false;

        let win_w = (ctx.screen_rect().width() - 40.0).clamp(560.0, widgets::PANEL_LG);
        let content_min = (win_w - 220.0).max(420.0);
        let win_h = (ctx.screen_rect().height() - 40.0).clamp(360.0, 640.0);
        let scroll_h = (win_h - 100.0).clamp(300.0, 560.0);

        egui::Window::new(
            RichText::new("Configuración")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .default_width(win_w)
        .default_height(win_h)
        .min_width(560.0)
        .resizable(true)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = theme::space::SM;
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(120.0);
                        for (tab, label) in [
                            (ConfigTab::Lectura, "Lectura"),
                            (ConfigTab::Biblioteca, "Biblioteca"),
                            (ConfigTab::Anotaciones, "Anotaciones"),
                            (ConfigTab::Consultas, "Consultas"),
                            (ConfigTab::Privacidad, "Privacidad"),
                            (ConfigTab::Avanzado, "Avanzado"),
                        ] {
                            let selected = self.config_tab == tab;
                            let mut text = RichText::new(label).size(theme::text::BODY);
                            if selected {
                                text = text.strong();
                            }
                            if ui.selectable_label(selected, text).clicked() {
                                self.config_tab = tab;
                            }
                        }
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        egui::ScrollArea::vertical()
                            .max_height(scroll_h)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.set_min_width(content_min);
                                    match self.config_tab {
                                ConfigTab::Lectura => {
                                    widgets::section_header(ui, th, "Lectura");
                                    ui.label(
                                        RichText::new(
                                            "Tipografía global. Para un libro concreto usá el panel Aa.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Tema").size(theme::text::LABEL).color(th.dim));
                                        egui::ComboBox::from_id_salt("config-theme")
                                            .selected_text(
                                                THEMES
                                                    .get(self.cfg.settings.theme)
                                                    .map(|t| t.name)
                                                    .unwrap_or("Tema"),
                                            )
                                            .show_ui(ui, |ui| {
                                                for (index, theme) in THEMES.iter().enumerate() {
                                                    if ui
                                                        .selectable_value(
                                                            &mut self.cfg.settings.theme,
                                                            index,
                                                            theme.name,
                                                        )
                                                        .clicked()
                                                    {
                                                        changed = true;
                                                    }
                                                }
                                            });
                                    });
                                    changed |= ui
                                        .add(
                                            egui::Slider::new(
                                                &mut self.cfg.settings.font_size,
                                                config::FONT_MIN..=config::FONT_MAX,
                                            )
                                            .text("tamaño")
                                            .step_by(0.5),
                                        )
                                        .changed();
                                    changed |= ui
                                        .add(
                                            egui::Slider::new(
                                                &mut self.cfg.settings.line_height,
                                                config::LINE_MIN..=config::LINE_MAX,
                                            )
                                            .text("interlineado")
                                            .step_by(0.02),
                                        )
                                        .changed();
                                    changed |= ui
                                        .add(
                                            egui::Slider::new(
                                                &mut self.cfg.settings.column_width,
                                                config::COLUMN_MIN..=config::COLUMN_MAX,
                                            )
                                            .text("ancho de columna")
                                            .step_by(10.0),
                                        )
                                        .changed();
                                    changed |= ui
                                        .add(
                                            egui::Slider::new(
                                                &mut self.cfg.settings.paragraph_space,
                                                config::PARAGRAPH_MIN..=config::PARAGRAPH_MAX,
                                            )
                                            .text("espacio entre párrafos")
                                            .step_by(0.05),
                                        )
                                        .changed();
                                    ui.add_space(6.0);
                                    if self.book.is_some() {
                                        if widgets::button(ui, th, "Tipografía de este libro…", widgets::ButtonKind::Secondary).clicked() {
                                            open_typography = true;
                                        }
                                    } else {
                                        ui.label(
                                            RichText::new(
                                                "Abrí un libro para editar su tipografía.",
                                            )
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        );
                                    }
                                }
                                ConfigTab::Biblioteca => {
                                    widgets::section_header(ui, th, "Biblioteca");
                                    ui.label(
                                        RichText::new(format!(
                                            "{book_count} libros · {missing} faltantes · {} carpetas",
                                            folders.len()
                                        ))
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        widgets::label(ui, th, "Al iniciar");
                                        let before = self.cfg.startup;
                                        egui::ComboBox::from_id_salt("startup-mode")
                                            .selected_text(self.cfg.startup.label())
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.cfg.startup,
                                                    config::StartupMode::LastBook,
                                                    "Último libro abierto",
                                                );
                                                ui.selectable_value(
                                                    &mut self.cfg.startup,
                                                    config::StartupMode::Library,
                                                    "Biblioteca",
                                                );
                                            });
                                        if before != self.cfg.startup {
                                            changed = true;
                                        }
                                    });
                                    ui.label(
                                        RichText::new(
                                            "Por defecto abre el último libro; si falta, abre la biblioteca.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.separator();
                                    for folder in &folders {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(truncate(folder, 64))
                                                    .size(theme::text::META)
                                                    .color(th.fg),
                                            );
                                            if widgets::button(
                                                ui,
                                                th,
                                                "×",
                                                widgets::ButtonKind::Ghost,
                                            )
                                            .on_hover_text("Dejar de vigilar")
                                            .clicked()
                                            {
                                                remove_folder = Some(folder.clone());
                                            }
                                        });
                                    }
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        if widgets::button(ui, th, "Añadir carpeta…", widgets::ButtonKind::Primary).clicked() {
                                            add_folder = true;
                                        }
                                        if widgets::button(ui, th, "Refrescar", widgets::ButtonKind::Secondary).clicked() {
                                            rescan = true;
                                        }
                                    });
                                }
                                ConfigTab::Anotaciones => {
                                    widgets::section_header(ui, th, "Anotaciones");
                                    ui.label(
                                        RichText::new(format!(
                                            "{annotation_count} destacados/notas guardados."
                                        ))
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.add_space(4.0);
                                    if self.book.is_some() {
                                        if widgets::button(ui, th, "Abrir panel de destacados", widgets::ButtonKind::Secondary).clicked() {
                                            open_annotations = true;
                                        }
                                    } else {
                                        ui.label(
                                            RichText::new(
                                                "Abrí un libro para ver sus destacados.",
                                            )
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        );
                                    }
                                    if widgets::button(ui, th, "Exportar todas a Markdown…", widgets::ButtonKind::Secondary).clicked() {
                                        export_all = true;
                                    }
                                    ui.label(
                                        RichText::new(
                                            "El historial de consultas se guarda aparte, en consultas.json.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                }
                                ConfigTab::Consultas => {
                                    widgets::section_header(ui, th, "Consultas");
                                    if ui
                                        .checkbox(
                                            &mut self.cfg.consulta.online_enabled,
                                            "Permitir consultas online",
                                        )
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                    ui.label(
                                        RichText::new(
                                            "Apagado por defecto. Al activarlo, solo se envía el texto que consultás.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.label(
                                        RichText::new(
                                            "La traducción con Ollama local funciona sin activar las consultas online.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new("Diccionario: Wiktionary")
                                            .size(theme::text::LABEL)
                                            .color(th.fg),
                                    );
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("Traducción").size(theme::text::LABEL).color(th.dim),
                                        );
                                        let before = self.cfg.consulta.translation_provider.clone();
                                        egui::ComboBox::from_id_salt("config-translation")
                                            .selected_text(
                                                if self.cfg.consulta.translation_provider
                                                    == "ollama"
                                                {
                                                    "Ollama (local)"
                                                } else {
                                                    "MyMemory (gratuita)"
                                                },
                                            )
                                            .show_ui(ui, |ui| {
                                                if ui
                                                    .selectable_value(
                                                        &mut self.cfg.consulta.translation_provider,
                                                        "mymemory".into(),
                                                        "MyMemory (gratuita)",
                                                    )
                                                    .clicked()
                                                {
                                                    changed = true;
                                                }
                                                if ui
                                                    .selectable_value(
                                                        &mut self.cfg.consulta.translation_provider,
                                                        "ollama".into(),
                                                        "Ollama (local)",
                                                    )
                                                    .clicked()
                                                {
                                                    if self
                                                        .cfg
                                                        .consulta
                                                        .translation_model
                                                        .trim()
                                                        .is_empty()
                                                    {
                                                        self.cfg.consulta.translation_model =
                                                            ai::provider("ollama")
                                                                .default_model
                                                                .into();
                                                    }
                                                    changed = true;
                                                }
                                            });
                                        if before != self.cfg.consulta.translation_provider {
                                            changed = true;
                                        }
                                    });
                                    if self.cfg.consulta.translation_provider == "ollama" {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new("Modelo local")
                                                    .size(theme::text::LABEL)
                                                    .color(th.dim),
                                            );
                                            if ui
                                                .add(
                                                    egui::TextEdit::singleline(
                                                        &mut self.cfg.consulta.translation_model,
                                                    )
                                                    .hint_text("llama3.2")
                                                    .desired_width(180.0),
                                                )
                                                .changed()
                                            {
                                                changed = true;
                                            }
                                        });
                                        ui.label(
                                            RichText::new(
                                                "Ollama debe estar abierto. La traducción no sale de tu Mac.",
                                            )
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new(
                                                "MyMemory admite hasta 500 caracteres; se recorta en palabra.",
                                            )
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        );
                                    }
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("Idioma destino").size(theme::text::LABEL).color(th.dim),
                                        );
                                        let before = self.cfg.consulta.target_lang.clone();
                                        egui::ComboBox::from_id_salt("config-target-lang")
                                            .selected_text(
                                                self.cfg
                                                    .consulta
                                                    .target_lang
                                                    .as_deref()
                                                    .map(str::to_uppercase)
                                                    .unwrap_or_else(|| "Auto".into()),
                                            )
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.cfg.consulta.target_lang,
                                                    None,
                                                    "Auto (en→es, es→en)",
                                                );
                                                for lang in ["es", "en", "pt", "fr", "de", "it"] {
                                                    ui.selectable_value(
                                                        &mut self.cfg.consulta.target_lang,
                                                        Some(lang.into()),
                                                        lang,
                                                    );
                                                }
                                            });
                                        if before != self.cfg.consulta.target_lang {
                                            changed = true;
                                        }
                                    });
                                    ui.add_space(theme::space::XS);
                                    widgets::label(ui, th, "IA · proveedor");
                                    ui.horizontal(|ui| {
                                        let before_provider = self.cfg.consulta.ai_provider.clone();
                                        egui::ComboBox::from_id_salt("config-ai-provider")
                                            .width(220.0)
                                            .selected_text(
                                                ai::provider(&self.cfg.consulta.ai_provider).label,
                                            )
                                            .show_ui(ui, |ui| {
                                                for provider in ai::ALL {
                                                    ui.selectable_value(
                                                        &mut self.cfg.consulta.ai_provider,
                                                        provider.id.into(),
                                                        provider.label,
                                                    );
                                                }
                                            });
                                        if before_provider != self.cfg.consulta.ai_provider {
                                            let provider =
                                                ai::provider(&self.cfg.consulta.ai_provider);
                                            self.cfg.consulta.ai_base_url =
                                                provider.base_url.into();
                                            self.cfg.consulta.ai_model =
                                                provider.default_model.into();
                                            self.ai_key =
                                                secrets::load(&self.cfg.consulta.ai_provider);
                                            self.ai_key_input.clear();
                                            self.confirm_delete_ai_key = false;
                                            changed = true;
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        widgets::label(ui, th, "API key");
                                        ui.add(
                                            egui::TextEdit::singleline(&mut self.ai_key_input)
                                                .password(true)
                                                .hint_text("sk-…")
                                                .desired_width(280.0),
                                        );
                                        if widgets::button(
                                            ui,
                                            th,
                                            "Guardar",
                                            widgets::ButtonKind::Secondary,
                                        )
                                        .clicked()
                                        {
                                            save_ai_key = true;
                                        }
                                        if self.ai_key.source == secrets::Source::Keyring
                                            && widgets::button(
                                                ui,
                                                th,
                                                "Borrar",
                                                widgets::ButtonKind::Destructive,
                                            )
                                            .clicked()
                                        {
                                            self.confirm_delete_ai_key = true;
                                        }
                                    });
                                    let key_status = if !ai::requires_key(&self.cfg.consulta.ai_provider)
                                        && self.ai_key.key.is_none()
                                    {
                                        "Ollama local no requiere API key".to_string()
                                    } else {
                                        secrets::status(&self.ai_key)
                                    };
                                    ui.label(
                                        RichText::new(key_status)
                                            .size(theme::text::META)
                                            .color(if self.ai_key.error.is_some() {
                                                th.error
                                            } else {
                                                th.dim
                                            }),
                                    );
                                    if ai::requires_key(&self.cfg.consulta.ai_provider) {
                                        ui.label(
                                            RichText::new(
                                                "Cada proveedor guarda su propia clave en el llavero.",
                                            )
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        );
                                    }
                                    if self.confirm_delete_ai_key
                                        && self.ai_key.source == secrets::Source::Keyring
                                    {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new("¿Borrar la clave del llavero?")
                                                    .size(theme::text::META)
                                                    .color(th.dim),
                                            );
                                            if widgets::button(
                                                ui,
                                                th,
                                                "Confirmar",
                                                widgets::ButtonKind::Destructive,
                                            )
                                            .clicked()
                                            {
                                                delete_ai_key = true;
                                                self.confirm_delete_ai_key = false;
                                            }
                                            if widgets::button(
                                                ui,
                                                th,
                                                "Cancelar",
                                                widgets::ButtonKind::Secondary,
                                            )
                                            .clicked()
                                            {
                                                self.confirm_delete_ai_key = false;
                                            }
                                        });
                                    }
                                    ui.horizontal(|ui| {
                                        widgets::label(ui, th, "base URL");
                                        changed |= ui
                                            .add(
                                                egui::TextEdit::singleline(
                                                    &mut self.cfg.consulta.ai_base_url,
                                                )
                                                .desired_width(450.0),
                                            )
                                            .changed();
                                    });
                                    ui.horizontal(|ui| {
                                        widgets::label(ui, th, "modelo");
                                        changed |= ui
                                            .add(
                                                egui::TextEdit::singleline(
                                                    &mut self.cfg.consulta.ai_model,
                                                )
                                                .desired_width(300.0),
                                            )
                                            .changed();
                                    });
                                    ui.horizontal(|ui| {
                                        widgets::label(ui, th, "timeout (s)");
                                        changed |= ui
                                            .add(
                                                egui::DragValue::new(
                                                    &mut self.cfg.consulta.ai_timeout_secs,
                                                )
                                                .range(5..=120),
                                            )
                                            .changed();
                                        widgets::label(ui, th, "máx. tokens");
                                        changed |= ui
                                            .add(
                                                egui::DragValue::new(
                                                    &mut self.cfg.consulta.ai_max_tokens,
                                                )
                                                .range(64..=4096),
                                            )
                                            .changed();
                                    });
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("Historial").size(theme::text::LABEL).color(th.dim),
                                        );
                                        changed |= ui
                                            .add(
                                                egui::DragValue::new(
                                                    &mut self.cfg.consulta.history_limit,
                                                )
                                                .range(0..=2000)
                                                .suffix(" entradas"),
                                            )
                                            .changed();
                                    });
                                    ui.label(
                                        RichText::new(format!(
                                            "{history_count} consultas guardadas · 0 = no guardar"
                                        ))
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    if self.config_confirm_clear {
                                        ui.horizontal(|ui| {
                                            ui.label("¿Borrar el historial?");
                                            if widgets::button(ui, th, "Confirmar", widgets::ButtonKind::Destructive).clicked() {
                                                clear_history = true;
                                            }
                                            if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked() {
                                                self.config_confirm_clear = false;
                                            }
                                        });
                                    } else if widgets::button(ui, th, "Limpiar historial", widgets::ButtonKind::Destructive).clicked() {
                                        self.config_confirm_clear = true;
                                    }
                                }
                                ConfigTab::Privacidad => {
                                    widgets::section_header(ui, th, "Privacidad");
                                    ui.label(
                                        RichText::new(
                                            "Nada sale a la red por seleccionar texto. Al consultar, \
                                             se envía el texto elegido (y el párrafo contenedor para IA) \
                                             al proveedor que corresponds. No se envían notas, rutas, \
                                             historial ni el EPUB completo.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.fg),
                                    );
                                    if ui
                                        .checkbox(
                                            &mut self.cfg.consulta.online_enabled,
                                            "Permitir consultas online",
                                        )
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                    ui.label(
                                        RichText::new(
                                            "El historial es local y se puede borrar. Las claves van \
                                             solo en OPENCODE_API_KEY.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    if self.config_confirm_clear {
                                        ui.horizontal(|ui| {
                                            ui.label("¿Borrar el historial?");
                                            if widgets::button(ui, th, "Confirmar", widgets::ButtonKind::Destructive).clicked() {
                                                clear_history = true;
                                            }
                                            if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked() {
                                                self.config_confirm_clear = false;
                                            }
                                        });
                                    } else if widgets::button(ui, th, "Limpiar historial", widgets::ButtonKind::Destructive).clicked() {
                                        self.config_confirm_clear = true;
                                    }
                                }
                                ConfigTab::Avanzado => {
                                    widgets::section_header(ui, th, "Avanzado");
                                    for (label, path) in [
                                        ("config.json", Config::file()),
                                        ("library.json", Library::file()),
                                        ("annotations.json", Annotations::file()),
                                        ("consultas.json", ConsultaHistory::file()),
                                    ] {
                                        ui.label(
                                            RichText::new(format!(
                                                "{label}: {}",
                                                truncate(&path.to_string_lossy(), 70)
                                            ))
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        );
                                    }
                                    if widgets::button(ui, th, "Abrir carpeta de datos", widgets::ButtonKind::Secondary).clicked() {
                                        open_data = true;
                                    }
                                    ui.add_space(theme::space::SM);
                                    ui.separator();
                                    widgets::section_header(ui, th, "Datos");
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(
                                                "Respaldo local de biblioteca, progreso, anotaciones, \
                                                 historial y ajustes. No incluye EPUB, portadas ni claves.",
                                            )
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        )
                                        .wrap(),
                                    );
                                    ui.label(
                                        RichText::new(
                                            "Importar reemplaza los datos actuales; antes se guarda un backup.",
                                        )
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.horizontal(|ui| {
                                        if widgets::button(
                                            ui,
                                            th,
                                            "Exportar datos…",
                                            widgets::ButtonKind::Secondary,
                                        )
                                        .clicked()
                                        {
                                            export_data = true;
                                        }
                                        if widgets::button(
                                            ui,
                                            th,
                                            "Importar datos…",
                                            widgets::ButtonKind::Secondary,
                                        )
                                        .clicked()
                                        {
                                            import_data = true;
                                        }
                                    });
                                    ui.add_space(theme::space::SM);
                                    ui.label(
                                        RichText::new(format!(
                                            "Candil {}",
                                            env!("CARGO_PKG_VERSION")
                                        ))
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                    ui.add_space(theme::space::SM);
                                    ui.separator();
                                    widgets::section_header(ui, th, "Actualizaciones");
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(
                                                "Consulta la API pública de GitHub sólo cuando lo pedís. \
                                                 No envía datos del libro ni de la biblioteca.",
                                            )
                                            .size(theme::text::META)
                                            .color(th.dim),
                                        )
                                        .wrap(),
                                    );
                                    let update_busy = self.update_rx.is_some();
                                    let update_state = self.update_state.clone();
                                    match &update_state {
                                        UpdateState::Idle => {}
                                        UpdateState::Checking => {
                                            ui.label(
                                                RichText::new("Buscando actualizaciones…")
                                                    .size(theme::text::BODY_SMALL)
                                                    .color(th.dim),
                                            );
                                        }
                                        UpdateState::UpToDate => {
                                            ui.label(
                                                RichText::new(format!(
                                                    "Estás al día (v{}).",
                                                    update::current_version()
                                                ))
                                                .size(theme::text::BODY_SMALL)
                                                .color(th.fg),
                                            );
                                        }
                                        UpdateState::Available { release, error } => {
                                            ui.label(
                                                RichText::new(format!(
                                                    "Nueva versión v{}",
                                                    release.version
                                                ))
                                                .strong()
                                                .color(th.fg),
                                            );
                                            if let Some(date) = release
                                                .published_at
                                                .split('T')
                                                .next()
                                                .filter(|date| !date.is_empty())
                                            {
                                                ui.label(
                                                    RichText::new(date)
                                                        .size(theme::text::META)
                                                        .color(th.dim),
                                                );
                                            }
                                            if !release.notes.is_empty() {
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(truncate(
                                                            &release.notes,
                                                            400,
                                                        ))
                                                        .size(theme::text::BODY_SMALL)
                                                        .color(th.fg),
                                                    )
                                                    .wrap(),
                                                );
                                            }
                                            if let Some(error) = error {
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(error)
                                                            .size(theme::text::META)
                                                            .color(th.error),
                                                    )
                                                    .wrap(),
                                                );
                                            }
                                            ui.horizontal(|ui| {
                                                if release.asset.is_none() {
                                                    ui.label(
                                                        RichText::new(
                                                            "No hay binario para esta plataforma.",
                                                        )
                                                        .size(theme::text::META)
                                                        .color(th.dim),
                                                    );
                                                } else if release.sums_url.is_none() {
                                                    ui.label(
                                                        RichText::new(
                                                            "El release no publica SHA256SUMS.txt; no se puede verificar.",
                                                        )
                                                        .size(theme::text::META)
                                                        .color(th.dim),
                                                    );
                                                } else if widgets::button(
                                                    ui,
                                                    th,
                                                    "Descargar",
                                                    widgets::ButtonKind::Primary,
                                                )
                                                .clicked()
                                                    && !update_busy
                                                {
                                                    download_update = Some(release.clone());
                                                }
                                                if widgets::button(
                                                    ui,
                                                    th,
                                                    "Abrir release",
                                                    widgets::ButtonKind::Secondary,
                                                )
                                                .clicked()
                                                {
                                                    open_release =
                                                        Some(release.page_url.clone());
                                                }
                                            });
                                        }
                                        UpdateState::Downloading(release) => {
                                            ui.label(
                                                RichText::new(format!(
                                                    "Descargando Candil v{} y verificando el checksum…",
                                                    release.version
                                                ))
                                                .size(theme::text::BODY_SMALL)
                                                .color(th.dim),
                                            );
                                        }
                                        UpdateState::Downloaded { release, path } => {
                                            ui.label(
                                                RichText::new(format!(
                                                    "Candil v{} descargado y verificado.",
                                                    release.version
                                                ))
                                                .size(theme::text::BODY_SMALL)
                                                .color(th.fg),
                                            );
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(truncate(
                                                        &path.to_string_lossy(),
                                                        90,
                                                    ))
                                                    .size(theme::text::META)
                                                    .color(th.dim),
                                                )
                                                .wrap(),
                                            );
                                            ui.horizontal(|ui| {
                                                if widgets::button(
                                                    ui,
                                                    th,
                                                    "Mostrar en Finder/Explorer",
                                                    widgets::ButtonKind::Secondary,
                                                )
                                                .clicked()
                                                {
                                                    reveal_download = Some(path.clone());
                                                }
                                                if widgets::button(
                                                    ui,
                                                    th,
                                                    "Abrir release",
                                                    widgets::ButtonKind::Ghost,
                                                )
                                                .clicked()
                                                {
                                                    open_release =
                                                        Some(release.page_url.clone());
                                                }
                                            });
                                        }
                                        UpdateState::Error(error) => {
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(error)
                                                        .size(theme::text::META)
                                                        .color(th.error),
                                                )
                                                .wrap(),
                                            );
                                        }
                                    }
                                    if !update_busy {
                                        let label = if matches!(
                                            &update_state,
                                            UpdateState::Idle | UpdateState::Error(_)
                                        ) {
                                            "Buscar actualizaciones"
                                        } else {
                                            "Buscar de nuevo"
                                        };
                                        if widgets::button(
                                            ui,
                                            th,
                                            label,
                                            widgets::ButtonKind::Secondary,
                                        )
                                        .clicked()
                                        {
                                            check_update = true;
                                        }
                                    }
                                    ui.add_space(theme::space::SM);
                                    if self.config_confirm_reset {
                                        ui.horizontal(|ui| {
                                            ui.label("¿Restablecer tipografía y consultas?");
                                            if widgets::button(ui, th, "Confirmar", widgets::ButtonKind::Destructive).clicked() {
                                                reset_config = true;
                                            }
                                            if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked() {
                                                self.config_confirm_reset = false;
                                            }
                                        });
                                    } else if widgets::button(
                                        ui,
                                        th,
                                        "Restablecer configuración",
                                        widgets::ButtonKind::Destructive,
                                    )
                                    .on_hover_text("No borra biblioteca, anotaciones ni historial")
                                    .clicked()
                                    {
                                        self.config_confirm_reset = true;
                                    }
                                    }
                                }
                                });
                            });
                        });
                    });
            });

        self.show_config = open;
        if was_open && !open {
            self.cfg.settings.clamp();
            self.cfg.consulta.clamp();
            let before = self.consulta_history.items.len();
            self.consulta_history.trim(self.cfg.consulta.history_limit);
            if self.consulta_history.items.len() != before {
                let _ = self.consulta_history.save();
            }
            self.dirty = true;
        }
        if changed {
            self.dirty = true;
        }
        if !self.cfg.consulta.online_enabled && self.consulta_busy {
            let local = self
                .consulta_pending_request
                .as_ref()
                .is_some_and(|request| match request.kind {
                    ConsultaKind::Traduccion => self.cfg.consulta.translation_provider == "ollama",
                    ConsultaKind::Ia => self.cfg.consulta.ai_provider == "ollama",
                    ConsultaKind::Diccionario => false,
                });
            if !local {
                self.consulta_rx = None;
                self.consulta_busy = false;
                self.consulta_pending_request = None;
                self.consulta_pending_prompt = None;
            }
        }
        if add_folder {
            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                let stats = self.library.scan_folder(&dir);
                if let Err(e) = self.library.save() {
                    self.status = Some((
                        format!("No pude guardar la biblioteca: {e}"),
                        Instant::now(),
                    ));
                } else {
                    self.status = Some((
                        format!("{} libros encontrados", stats.found),
                        Instant::now(),
                    ));
                }
            }
        }
        if rescan {
            let stats = self.library.rescan();
            if let Err(e) = self.library.save() {
                self.status = Some((
                    format!("No pude guardar la biblioteca: {e}"),
                    Instant::now(),
                ));
            } else {
                self.status = Some((
                    format!("{} libros, {} actualizados", stats.found, stats.updated),
                    Instant::now(),
                ));
            }
        }
        if let Some(folder) = remove_folder {
            self.library.unwatch_folder(&folder);
            if let Err(e) = self.library.save() {
                self.status = Some((
                    format!("No pude guardar la biblioteca: {e}"),
                    Instant::now(),
                ));
            }
        }
        if open_typography {
            self.show_settings = true;
            self.show_config = false;
        }
        if open_annotations {
            self.show_annotations = true;
            self.annotations_global = false;
            self.show_config = false;
        }
        if export_all {
            let previous = self.annotations_global;
            self.annotations_global = true;
            self.export_annotations();
            self.annotations_global = previous;
        }
        if open_data {
            open_path(&Config::data_dir().to_string_lossy());
        }
        if save_ai_key {
            match secrets::save(&self.cfg.consulta.ai_provider, &self.ai_key_input) {
                Ok(()) => {
                    self.ai_key = secrets::load(&self.cfg.consulta.ai_provider);
                    self.ai_key_input.clear();
                    self.say("Clave guardada en el llavero.");
                }
                Err(error) => {
                    self.ai_key.error = Some(error.clone());
                    self.say(error);
                }
            }
        }
        if delete_ai_key {
            match secrets::delete(&self.cfg.consulta.ai_provider) {
                Ok(()) => {
                    self.ai_key = secrets::load(&self.cfg.consulta.ai_provider);
                    self.ai_key_input.clear();
                    self.say("Clave borrada del llavero.");
                }
                Err(error) => {
                    self.ai_key.error = Some(error.clone());
                    self.say(error);
                }
            }
        }
        if clear_history {
            self.consulta_history.clear();
            if let Err(e) = self.consulta_history.save() {
                self.status = Some((format!("No pude limpiar el historial: {e}"), Instant::now()));
            } else {
                self.say("Historial borrado.");
            }
            self.config_confirm_clear = false;
        }
        if check_update {
            self.check_for_updates();
        }
        if let Some(release) = download_update {
            self.download_update(release);
        }
        if let Some(url) = open_release {
            open_external(&url);
        }
        if let Some(path) = reveal_download {
            reveal_path(&path);
        }
        if export_data {
            self.export_data();
        }
        if import_data {
            self.import_data(ctx);
        }
        if reset_config {
            self.cfg.settings = Settings::default();
            self.cfg.consulta = config::ConsultaSettings::default();
            self.cfg.settings.clamp();
            self.cfg.consulta.clamp();
            self.active_color = 0;
            self.dirty = true;
            self.config_confirm_reset = false;
            self.status = Some(("Configuración restablecida.".into(), Instant::now()));
        }
        if !self.show_config {
            self.config_confirm_clear = false;
            self.config_confirm_reset = false;
        }
    }

    fn run_consulta(&mut self, kind: ConsultaKind) {
        if self.consulta_busy {
            return;
        }
        let local = match kind {
            ConsultaKind::Traduccion => self.cfg.consulta.translation_provider == "ollama",
            ConsultaKind::Ia => self.cfg.consulta.ai_provider == "ollama",
            ConsultaKind::Diccionario => false,
        };
        if !self.cfg.consulta.online_enabled && !local {
            self.consulta_error = Some(
                "Las consultas online están desactivadas. Activalas o usá traducción local con Ollama."
                    .into(),
            );
            return;
        }
        let Some(target) = self.consulta_target.clone() else {
            return;
        };
        if target.text.trim().is_empty() {
            self.consulta_error = Some("No hay texto seleccionado.".into());
            return;
        }
        let target_lang = consulta::target_lang(&self.cfg.consulta, &target.source_lang);
        let id = self.consulta_id + 1;
        self.consulta_id = id;
        let request = ConsultaRequest {
            id,
            kind,
            text: target.text.clone(),
            context: target.context.clone(),
            source_lang: target.source_lang.clone(),
            target_lang,
            book: target.book.clone(),
            title: target.title.clone(),
            author: target.author.clone(),
        };
        let settings = self.cfg.consulta.clone();
        let prompt = self.consulta_prompt;
        let session = self.consulta_session.clone();
        let key = self.ai_key.key.clone();
        let (tx, rx) = mpsc::channel();
        self.consulta_rx = Some(rx);
        self.consulta_busy = true;
        self.consulta_result = None;
        self.consulta_error = None;
        self.consulta_pending_request = Some(request.clone());
        self.consulta_pending_prompt = Some(prompt);
        std::thread::spawn(move || {
            let transport = consulta::HttpTransport::new(settings.ai_timeout_secs);
            let result = consulta::run(
                &transport,
                &request,
                &settings,
                prompt,
                key.as_deref(),
                &session,
            );
            let _ = tx.send((id, result));
        });
    }

    fn poll_consulta(&mut self) {
        let received = self.consulta_rx.as_ref().map(|rx| rx.try_recv());
        let Some(received) = received else {
            return;
        };
        match received {
            Ok((id, result)) => {
                if id != self.consulta_id {
                    return;
                }
                self.consulta_rx = None;
                self.consulta_busy = false;
                let target = self.consulta_target.clone();
                let prompt = self.consulta_pending_prompt.take();
                match result {
                    Ok(response) => {
                        if let Some(request) = self.consulta_pending_request.take() {
                            let mut entry = consulta::entry_from_response(&request, &response);
                            if let Some(target) = &target {
                                entry.chapter = target.chapter;
                                entry.block = target.block;
                            }
                            if let Some(prompt) = prompt {
                                entry.prompt = Some(prompt.label().into());
                            }
                            self.push_history(entry);
                        }
                        self.consulta_result = Some(response);
                        self.consulta_error = None;
                    }
                    Err(error) => {
                        if let Some(request) = self.consulta_pending_request.take() {
                            let provider = match request.kind {
                                ConsultaKind::Diccionario => "Wiktionary",
                                ConsultaKind::Traduccion
                                    if self.cfg.consulta.translation_provider == "ollama" =>
                                {
                                    "Ollama (local)"
                                }
                                ConsultaKind::Traduccion => "MyMemory",
                                ConsultaKind::Ia => {
                                    ai::provider(&self.cfg.consulta.ai_provider).label
                                }
                            };
                            let mut entry = consulta::entry_from_error(&request, provider, &error);
                            if let Some(target) = &target {
                                entry.chapter = target.chapter;
                                entry.block = target.block;
                            }
                            if let Some(prompt) = prompt {
                                entry.prompt = Some(prompt.label().into());
                            }
                            self.push_history(entry);
                        }
                        self.consulta_error = Some(error);
                        self.consulta_result = None;
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                self.consulta_rx = None;
                self.consulta_busy = false;
                self.consulta_pending_request = None;
                self.consulta_pending_prompt = None;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    fn push_history(&mut self, entry: ConsultaEntry) {
        let limit = self.cfg.consulta.history_limit;
        if limit == 0 {
            return;
        }
        self.consulta_history.push(entry, limit);
        if let Err(e) = self.consulta_history.save() {
            self.status = Some((format!("No pude guardar el historial: {e}"), Instant::now()));
        }
    }

    fn consulta_provider_label(&self) -> &'static str {
        match self.consulta_tab {
            ConsultaTab::Diccionario => "Wiktionary",
            ConsultaTab::Traduccion if self.cfg.consulta.translation_provider == "ollama" => {
                "Ollama local"
            }
            ConsultaTab::Traduccion => "MyMemory",
            ConsultaTab::Ia => ai::provider(&self.cfg.consulta.ai_provider).label,
            ConsultaTab::Historial => "Historial",
        }
    }

    fn consulta_ui(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_consulta {
            return;
        }
        let target = self.consulta_target.clone();
        let has_key =
            self.ai_key.key.is_some() || !ai::requires_key(&self.cfg.consulta.ai_provider);
        let local_translation = self.cfg.consulta.translation_provider == "ollama";
        let local_ai = self.cfg.consulta.ai_provider == "ollama";
        let online_needed = match self.consulta_tab {
            ConsultaTab::Diccionario => true,
            ConsultaTab::Traduccion => !local_translation,
            ConsultaTab::Ia => !local_ai,
            ConsultaTab::Historial => false,
        };
        let mut open = self.show_consulta;
        let mut run_dictionary = false;
        let mut run_translation = false;
        let mut run_ai = false;
        let mut activate_online = false;
        let mut clear_history = false;
        let mut open_history: Option<ConsultaEntry> = None;
        let mut back_history = false;
        let mut jump_history: Option<ConsultaEntry> = None;
        let history_entries: Vec<ConsultaEntry> = self
            .consulta_history
            .newest_first()
            .into_iter()
            .cloned()
            .collect();
        let target_lang = target
            .as_ref()
            .map(|t| consulta::target_lang(&self.cfg.consulta, &t.source_lang))
            .unwrap_or_else(|| "es".into());

        egui::Window::new(
            RichText::new("Consulta")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_MD)
        .frame(widgets::panel_frame_fill(th, consulta_window_fill(th)))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = theme::space::SM;
            let Some(target) = &target else {
                ui.label(
                    RichText::new("Seleccioná una palabra o frase en el libro.")
                        .size(theme::text::BODY_SMALL)
                        .color(th.dim),
                );
                return;
            };
            ui.label(
                RichText::new(format!("“{}”", truncate(&target.text, 220)))
                    .size(theme::text::BODY)
                    .color(th.fg),
            );
            ui.label(
                RichText::new(format!(
                    "{} → {}",
                    target.source_lang.to_uppercase(),
                    target_lang.to_uppercase()
                ))
                .size(theme::text::META)
                .color(th.dim),
            );

            if !self.cfg.consulta.online_enabled && online_needed {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Consultas online desactivadas.")
                        .size(theme::text::LABEL)
                        .color(th.dim),
                );
                if widgets::button(
                    ui,
                    th,
                    "Activar consultas online",
                    widgets::ButtonKind::Primary,
                )
                .on_hover_text("Se enviará solo el texto que consultes, con tu confirmación")
                .clicked()
                {
                    activate_online = true;
                }
                ui.label(
                    RichText::new(
                        "Al activarlas, el texto seleccionado se envía al proveedor elegido. \
                             Nada sale sin que pulses un botón.",
                    )
                    .size(theme::text::META)
                    .color(th.dim),
                );
                if local_translation {
                    ui.label(
                        RichText::new(
                            "La traducción con Ollama local sigue disponible sin activar esto.",
                        )
                        .size(theme::text::META)
                        .color(th.dim),
                    );
                }
            }

            ui.horizontal(|ui| {
                for (tab, label) in [
                    (ConsultaTab::Diccionario, "Diccionario"),
                    (ConsultaTab::Traduccion, "Traducción"),
                    (ConsultaTab::Ia, "IA"),
                    (ConsultaTab::Historial, "Historial"),
                ] {
                    if ui
                        .selectable_label(
                            self.consulta_tab == tab,
                            RichText::new(label).size(theme::text::BODY),
                        )
                        .clicked()
                    {
                        self.consulta_tab = tab;
                        self.consulta_result = None;
                        self.consulta_error = None;
                        self.consulta_history_view = None;
                    }
                }
            });
            ui.separator();

            if self.consulta_tab == ConsultaTab::Traduccion {
                ui.horizontal(|ui| {
                    widgets::label(ui, th, "Destino");
                    let before = self.cfg.consulta.target_lang.clone();
                    egui::ComboBox::from_id_salt("consulta-target")
                        .selected_text(
                            self.cfg
                                .consulta
                                .target_lang
                                .as_deref()
                                .map(|s| s.to_uppercase())
                                .unwrap_or_else(|| "Auto".into()),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.cfg.consulta.target_lang, None, "Auto");
                            for lang in ["es", "en", "pt", "fr", "de", "it"] {
                                ui.selectable_value(
                                    &mut self.cfg.consulta.target_lang,
                                    Some(lang.into()),
                                    lang,
                                );
                            }
                        });
                    if before != self.cfg.consulta.target_lang {
                        self.dirty = true;
                    }
                });
            }

            match self.consulta_tab {
                ConsultaTab::Diccionario => {
                    if target.text.split_whitespace().count() > 1 {
                        ui.label(
                            RichText::new("El diccionario funciona mejor con una palabra.")
                                .size(theme::text::META)
                                .color(th.dim),
                        );
                    }
                    let enabled = self.cfg.consulta.online_enabled && !self.consulta_busy;
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new(
                                RichText::new("Buscar definición")
                                    .size(theme::text::BODY)
                                    .color(th.fg),
                            )
                            .min_size(Vec2::new(0.0, 30.0))
                            .corner_radius(theme::radius::MD),
                        )
                        .clicked()
                    {
                        run_dictionary = true;
                    }
                }
                ConsultaTab::Traduccion => {
                    let model = if self.cfg.consulta.translation_model.trim().is_empty() {
                        ai::provider("ollama").default_model
                    } else {
                        self.cfg.consulta.translation_model.trim()
                    };
                    if local_translation {
                        ui.label(
                            RichText::new(format!("Ollama local · {model}"))
                                .size(theme::text::META)
                                .color(th.dim),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "MyMemory · hasta {} caracteres",
                                consulta::MYMEMORY_MAX_CHARS
                            ))
                            .size(theme::text::META)
                            .color(th.dim),
                        );
                        if target.text.chars().count() > consulta::MYMEMORY_MAX_CHARS {
                            ui.label(
                                RichText::new(
                                    "Se traducirán los primeros 500 caracteres, cortando en palabra.",
                                )
                                .size(theme::text::META)
                                .color(th.dim),
                            );
                        }
                    }
                    let enabled = (self.cfg.consulta.online_enabled || local_translation)
                        && !self.consulta_busy;
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new(
                                RichText::new("Traducir")
                                    .size(theme::text::BODY)
                                    .color(th.fg),
                            )
                            .min_size(Vec2::new(0.0, 30.0))
                            .corner_radius(theme::radius::MD),
                        )
                        .clicked()
                    {
                        run_translation = true;
                    }
                }
                ConsultaTab::Ia => {
                    ui.horizontal(|ui| {
                        widgets::label(ui, th, "Prompt");
                        egui::ComboBox::from_id_salt("consulta-prompt")
                            .selected_text(self.consulta_prompt.label())
                            .show_ui(ui, |ui| {
                                for prompt in AiPrompt::ALL {
                                    ui.selectable_value(
                                        &mut self.consulta_prompt,
                                        prompt,
                                        prompt.label(),
                                    );
                                }
                            });
                    });
                    let provider = ai::provider(&self.cfg.consulta.ai_provider);
                    if !provider.needs_key && self.ai_key.key.is_none() {
                        widgets::meta(ui, th, "Ollama local no requiere API key");
                    } else if let Some(error) = &self.ai_key.error {
                        ui.label(RichText::new(error).size(theme::text::META).color(th.error));
                    } else if has_key {
                        ui.label(
                            RichText::new(secrets::status(&self.ai_key))
                                .size(theme::text::META)
                                .color(th.dim),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "Falta la API key de {}: guardala en Configuración.",
                                provider.label
                            ))
                            .size(theme::text::META)
                            .color(th.error),
                        );
                    }
                    ui.label(
                        RichText::new(if local_ai {
                            "Ollama corre en tu Mac; la selección no sale a internet.".to_string()
                        } else {
                            format!(
                                "Se enviará la selección y el párrafo contenedor a {}.",
                                provider.label
                            )
                        })
                        .size(theme::text::META)
                        .color(th.dim),
                    );
                    if target.text.chars().count() > consulta::MAX_TEXT_CHARS {
                        ui.label(
                            RichText::new(format!(
                                "La selección supera {} caracteres; se enviarán los primeros {}.",
                                consulta::MAX_TEXT_CHARS,
                                consulta::MAX_TEXT_CHARS
                            ))
                            .size(theme::text::META)
                            .color(th.dim),
                        );
                    }
                    let enabled = (self.cfg.consulta.online_enabled || local_ai)
                        && !self.consulta_busy
                        && has_key;
                    if ui
                        .add_enabled(
                            enabled,
                            egui::Button::new(
                                RichText::new("Analizar con IA")
                                    .size(theme::text::BODY)
                                    .color(th.fg),
                            )
                            .min_size(Vec2::new(0.0, 30.0))
                            .corner_radius(theme::radius::MD),
                        )
                        .clicked()
                    {
                        run_ai = true;
                    }
                }
                ConsultaTab::Historial => {
                    if let Some(view) = &self.consulta_history_view {
                        let mut meta = format!(
                            "{} · {} · {} · Cap. {}",
                            view.kind.label(),
                            view.provider,
                            truncate(&view.book_title, 32),
                            view.chapter + 1
                        );
                        if let Some(prompt) = &view.prompt {
                            meta.push_str(" · ");
                            meta.push_str(prompt);
                        }
                        ui.label(RichText::new(meta).size(theme::text::META).color(th.dim));
                        ui.label(
                            RichText::new(format!("“{}”", truncate(&view.quote, 220)))
                                .size(theme::text::BODY)
                                .color(th.fg),
                        );
                        if let Some(error) = &view.error {
                            ui.label(
                                RichText::new(error)
                                    .size(theme::text::LABEL)
                                    .color(th.error),
                            );
                        } else {
                            egui::ScrollArea::vertical()
                                .max_height(330.0)
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(&view.response)
                                            .size(theme::text::BODY_SMALL)
                                            .color(th.fg),
                                    );
                                });
                        }
                        ui.horizontal(|ui| {
                            if widgets::button(ui, th, "Ir al texto", widgets::ButtonKind::Primary)
                                .clicked()
                            {
                                jump_history = Some(view.clone());
                            }
                            if widgets::button(ui, th, "Copiar", widgets::ButtonKind::Ghost)
                                .clicked()
                            {
                                ctx.copy_text(view.response.clone());
                            }
                            if widgets::button(ui, th, "Volver", widgets::ButtonKind::Ghost)
                                .clicked()
                            {
                                back_history = true;
                            }
                        });
                    } else if history_entries.is_empty() {
                        ui.label(
                            RichText::new("Sin consultas guardadas.")
                                .size(theme::text::LABEL)
                                .color(th.dim),
                        );
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(330.0)
                            .show(ui, |ui| {
                                for entry in &history_entries {
                                    let label = format!(
                                        "{} · {} · {}",
                                        entry.kind.label(),
                                        truncate(&entry.quote, 50),
                                        human_age(entry.created)
                                    );
                                    if ui.selectable_label(false, label).clicked() {
                                        open_history = Some(entry.clone());
                                    }
                                    if let Some(error) = &entry.error {
                                        ui.label(
                                            RichText::new(truncate(error, 100))
                                                .size(theme::text::META)
                                                .color(th.error),
                                        );
                                    }
                                }
                            });
                        ui.separator();
                        if self.consulta_confirm_clear {
                            ui.label(
                                RichText::new("¿Borrar todo el historial?")
                                    .size(theme::text::META)
                                    .color(th.dim),
                            );
                            if widgets::button(
                                ui,
                                th,
                                "Confirmar",
                                widgets::ButtonKind::Destructive,
                            )
                            .clicked()
                            {
                                clear_history = true;
                            }
                            if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary)
                                .clicked()
                            {
                                self.consulta_confirm_clear = false;
                            }
                        } else if widgets::button(
                            ui,
                            th,
                            "Limpiar historial",
                            widgets::ButtonKind::Destructive,
                        )
                        .clicked()
                        {
                            self.consulta_confirm_clear = true;
                        }
                    }
                    ui.label(
                        RichText::new(format!(
                            "{} entradas · límite {}",
                            self.consulta_history.items.len(),
                            self.cfg.consulta.history_limit
                        ))
                        .size(theme::text::META)
                        .color(th.dim),
                    );
                }
            }

            ui.separator();
            if self.consulta_busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new("Consultando…")
                            .size(theme::text::LABEL)
                            .color(th.dim),
                    );
                });
            } else if let Some(error) = &self.consulta_error {
                ui.label(
                    RichText::new(error)
                        .size(theme::text::LABEL)
                        .color(th.error),
                );
            } else if let Some(result) = &self.consulta_result {
                ui.label(
                    RichText::new(format!(
                        "Fuente: {}{}",
                        result.provider,
                        result
                            .model
                            .as_deref()
                            .map(|m| format!(" · {m}"))
                            .unwrap_or_default()
                    ))
                    .size(theme::text::META)
                    .color(th.dim),
                );
                egui::ScrollArea::vertical()
                    .max_height(360.0)
                    .show(ui, |ui| {
                        for section in &result.sections {
                            ui.label(
                                RichText::new(&section.title)
                                    .size(theme::text::META)
                                    .color(th.dim),
                            );
                            ui.label(
                                RichText::new(&section.body)
                                    .size(theme::text::BODY_SMALL)
                                    .color(th.fg),
                            );
                            ui.add_space(3.0);
                        }
                    });
                ui.horizontal(|ui| {
                    if widgets::button(ui, th, "Copiar", widgets::ButtonKind::Secondary).clicked() {
                        ctx.copy_text(result.plain_text());
                    }
                    if let Some(url) = &result.source_url {
                        ui.hyperlink_to("Abrir fuente", url);
                    }
                });
            } else if self.consulta_tab != ConsultaTab::Historial {
                ui.label(
                    RichText::new(format!(
                        "Pulsá el botón para consultar {}.",
                        self.consulta_provider_label()
                    ))
                    .size(theme::text::META)
                    .color(th.dim),
                );
            }
        });

        self.show_consulta = open;
        if activate_online {
            self.cfg.consulta.online_enabled = true;
            self.dirty = true;
        }
        if let Some(entry) = open_history {
            self.consulta_history_view = Some(entry);
        }
        if back_history {
            self.consulta_history_view = None;
        }
        if let Some(entry) = jump_history {
            self.jump_to_consulta_history(&entry, ctx);
        }
        if clear_history {
            self.consulta_history.clear();
            if let Err(e) = self.consulta_history.save() {
                self.status = Some((format!("No pude limpiar el historial: {e}"), Instant::now()));
            } else {
                self.say("Historial borrado.");
            }
            self.consulta_confirm_clear = false;
        }
        if run_dictionary {
            self.run_consulta(ConsultaKind::Diccionario);
        }
        if run_translation {
            self.run_consulta(ConsultaKind::Traduccion);
        }
        if run_ai {
            self.run_consulta(ConsultaKind::Ia);
        }
        if !self.show_consulta {
            self.consulta_rx = None;
            self.consulta_busy = false;
            self.consulta_confirm_clear = false;
        }
    }

    fn jump_to_consulta_history(&mut self, entry: &ConsultaEntry, ctx: &egui::Context) {
        let target =
            std::fs::canonicalize(&entry.book).unwrap_or_else(|_| PathBuf::from(&entry.book));
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
        if entry.chapter != self.chapter {
            self.load_chapter(entry.chapter, None);
        }
        let Some((index, start, end)) =
            resolve_history_range(&self.blocks, entry.block, &entry.quote)
        else {
            self.status = Some((
                format!(
                    "No encontré «{}» en este capítulo.",
                    truncate(&entry.quote, 50)
                ),
                Instant::now(),
            ));
            return;
        };
        self.pending_restore = None;
        self.pending_block_jump = Some((index, entry.quote.clone()));
        self.pending_search_jump = Some((index, start));
        self.search_highlight = Some((entry.chapter, index, start, end));
        self.show_consulta = false;
        self.consulta_rx = None;
        self.consulta_busy = false;
        self.consulta_pending_request = None;
        self.consulta_pending_prompt = None;
        self.show_config = false;
        self.selection = None;
        self.note = None;
        self.back_spot = None;
        ctx.request_repaint();
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
        if !self.notes_query.trim().is_empty() {
            entries.retain(|(book, chapter, annotation)| {
                note_matches(
                    &self.notes_query,
                    &annotation.quote,
                    &annotation.note,
                    book,
                    chapter,
                )
            });
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
        self.back_spot = None;
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
        egui::Window::new(
            RichText::new("Buscar en el libro")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_MD)
        .frame(widgets::panel_frame(th))
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
                        .size(theme::text::META)
                        .color(th.dim),
                );
                return;
            }

            let total = self.search_results.len();
            if total == 0 {
                ui.label(
                    RichText::new("Sin coincidencias.")
                        .size(theme::text::LABEL)
                        .color(th.dim),
                );
                return;
            }

            let count = if self.search_truncated {
                format!("Mostrando los primeros {total}; afiná la búsqueda.",)
            } else {
                format!("{total} resultado(s)")
            };
            ui.label(RichText::new(count).size(theme::text::META).color(th.dim));

            let dim = TextFormat {
                font_id: FontId::new(theme::text::BODY_SMALL, fonts::family(false, false)),
                color: th.dim,
                ..Default::default()
            };
            let mut accent = dim.clone();
            accent.color = th.accent;

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for (index, hit) in self.search_results.iter().enumerate() {
                        let row_frame =
                            egui::Frame::new()
                                .inner_margin(theme::space::SM)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    let mut job = LayoutJob::default();
                                    job.wrap.max_width = ui.available_width();
                                    job.append(&hit.before, 0.0, dim.clone());
                                    job.append(&hit.matched, 0.0, accent.clone());
                                    job.append(&hit.after, 0.0, dim.clone());
                                    ui.add(egui::Label::new(job).wrap().sense(Sense::hover()));
                                    ui.label(
                                        RichText::new(format!(
                                            "Cap. {} · {}",
                                            hit.chapter + 1,
                                            truncate(
                                                titles.get(index).map(String::as_str).unwrap_or(""),
                                                44
                                            )
                                        ))
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                });
                        let id = ui.make_persistent_id(("search-row", index));
                        let response = ui.interact(row_frame.response.rect, id, Sense::click());
                        let active = index == self.search_active;
                        if response.has_focus() {
                            ui.painter().rect_stroke(
                                row_frame.response.rect,
                                theme::radius::SM,
                                Stroke::new(2.0, th.accent),
                                egui::StrokeKind::Inside,
                            );
                        } else if response.hovered() || active {
                            ui.painter().rect_stroke(
                                row_frame.response.rect,
                                theme::radius::SM,
                                Stroke::new(1.0, th.accent.gamma_multiply(0.55)),
                                egui::StrokeKind::Inside,
                            );
                        }
                        let activated = response.clicked()
                            || (response.has_focus()
                                && ui.input(|i| {
                                    i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)
                                }));
                        if activated {
                            jump = Some(index);
                        }
                        ui.add_space(theme::space::XS);
                    }
                });

            ui.add_space(2.0);
            ui.label(
                RichText::new("Enter: ir · n/N: siguiente/anterior · Esc: cerrar")
                    .size(theme::text::META)
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
            key_k,
            key_b,
            key_comma,
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
                i.key_pressed(Key::K),
                i.key_pressed(Key::B),
                i.key_pressed(Key::Comma),
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
            self.show_bookmarks = false;
            self.show_goto = false;
            self.selection = None;
            self.note = None;
            ctx.request_repaint();
        }

        if ctrl && key_comma {
            if self.show_config {
                self.show_config = false;
            } else {
                let tab = if self.book.is_some() {
                    ConfigTab::Consultas
                } else {
                    ConfigTab::Biblioteca
                };
                self.open_config(ctx, tab);
            }
        }

        if key_k && self.book.is_some() && self.note.is_none() && self.selection.is_some() {
            self.open_consulta(ctx);
            if shift && self.show_consulta {
                self.consulta_tab = ConsultaTab::Traduccion;
                self.consulta_result = None;
                self.consulta_error = None;
                self.run_consulta(ConsultaKind::Traduccion);
            }
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
                self.show_bookmarks = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
                self.selection = None;
            }
            ctx.request_repaint();
        }

        if key_b
            && !typing
            && self.book.is_some()
            && self.note.is_none()
            && self.selection.is_none()
        {
            if shift {
                self.show_bookmarks = !self.show_bookmarks;
                self.show_toc = false;
                self.show_settings = false;
                self.show_search = false;
                self.show_annotations = false;
                self.show_goto = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
                self.selection = None;
                ctx.request_repaint();
            } else {
                self.add_bookmark_here();
            }
        }

        if !typing
            && (ctx.input(|i| i.key_pressed(Key::F1))
                || ctx.input(|i| i.key_pressed(Key::Questionmark)))
        {
            self.show_help = !self.show_help;
            ctx.request_repaint();
        }

        if key_esc {
            if self.external_link.is_some() {
                self.external_link = None;
            } else if self.link_preview.is_some() {
                self.link_preview = None;
            } else if self.show_help {
                self.show_help = false;
            } else if self.note.is_some() {
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
            } else if self.show_bookmarks {
                self.show_bookmarks = false;
            } else if self.show_consulta {
                self.show_consulta = false;
                self.consulta_rx = None;
                self.consulta_busy = false;
                self.consulta_confirm_clear = false;
            } else if self.show_config {
                self.show_config = false;
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
                self.show_bookmarks = false;
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
            self.show_bookmarks = false;
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
            self.show_bookmarks = false;
            if self.show_settings {
                self.show_goto = false;
                self.show_search = false;
                self.search_highlight = None;
                self.pending_search_jump = None;
            }
        }
    }

    // -- pantallas -----------------------------------------------------------

    fn poll_update(&mut self, ctx: &egui::Context) {
        let received = self.update_rx.as_ref().map(|rx| rx.try_recv());
        let Some(received) = received else {
            return;
        };
        match received {
            Ok(UpdateMsg::Checked(result)) => {
                self.update_rx = None;
                self.update_state = match result {
                    Ok(Some(release)) => UpdateState::Available {
                        release,
                        error: None,
                    },
                    Ok(None) => UpdateState::UpToDate,
                    Err(error) => UpdateState::Error(error),
                };
                ctx.request_repaint();
            }
            Ok(UpdateMsg::Downloaded { release, result }) => {
                self.update_rx = None;
                self.update_state = match result {
                    Ok(path) => UpdateState::Downloaded { release, path },
                    Err(error) => UpdateState::Available {
                        release,
                        error: Some(error),
                    },
                };
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(120));
            }
            Err(TryRecvError::Disconnected) => {
                self.update_rx = None;
                self.update_state = UpdateState::Error("La comprobación se interrumpió.".into());
                ctx.request_repaint();
            }
        }
    }

    fn check_for_updates(&mut self) {
        if self.update_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.update_rx = Some(rx);
        self.update_state = UpdateState::Checking;
        std::thread::spawn(move || {
            let result = update::check_latest();
            let _ = tx.send(UpdateMsg::Checked(result));
        });
    }

    fn download_update(&mut self, release: update::ReleaseInfo) {
        if self.update_rx.is_some() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.update_rx = Some(rx);
        self.update_state = UpdateState::Downloading(release.clone());
        std::thread::spawn(move || {
            let dir = update::default_download_dir();
            let result = update::download_release(&release, &dir);
            let _ = tx.send(UpdateMsg::Downloaded { release, result });
        });
    }

    fn reload_user_data(&mut self) {
        self.cfg = Config::load();
        self.library = Library::load();
        self.annotations = Annotations::load().0;
        self.consulta_history = ConsultaHistory::load().0;
        self.library_textures.clear();
        self.applied_theme = None;
        self.dirty = false;
    }

    fn export_data(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name(backup::default_name())
            .add_filter("ZIP", &["zip"])
            .save_file()
        else {
            return;
        };
        match backup::export(&self.library.data_dir, &path) {
            Ok(result) => self.say(format!("Datos exportados a {}", result.path.display())),
            Err(error) => self.say(format!("No pude exportar: {error}")),
        }
    }

    fn import_data(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Respaldo de Candil", &["zip"])
            .pick_file()
        else {
            return;
        };
        match backup::import(&path, &self.library.data_dir) {
            Ok(result) => {
                self.reload_user_data();
                self.go_library();
                self.data_notice = Some(match result.backup {
                    Some(backup) => format!("Backup previo: {}", backup.display()),
                    None => "No había datos previos que respaldar.".to_string(),
                });
                if !self.library.folders.is_empty() {
                    self.start_scan(None, ctx);
                }
            }
            Err(error) => self.say(format!("No pude importar: {error}")),
        }
    }

    fn data_notice_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(notice) = self.data_notice.clone() else {
            return;
        };
        let mut close = false;
        let _ = egui::Modal::new(egui::Id::new("data-import-notice"))
            .backdrop_color(Color32::from_black_alpha(40))
            .frame(widgets::panel_frame(th))
            .show(ctx, |ui| {
                ui.set_width(440.0);
                ui.label(
                    RichText::new("Datos importados")
                        .size(theme::text::TITLE)
                        .strong()
                        .color(th.fg),
                );
                ui.add_space(theme::space::SM);
                ui.add(
                    egui::Label::new(
                        RichText::new(
                            "La biblioteca, el progreso y las anotaciones se recargaron.",
                        )
                        .size(theme::text::BODY)
                        .color(th.fg),
                    )
                    .wrap(),
                );
                ui.add(
                    egui::Label::new(RichText::new(&notice).size(theme::text::META).color(th.dim))
                        .wrap(),
                );
                ui.add_space(theme::space::SM);
                if widgets::button(ui, th, "Cerrar", widgets::ButtonKind::Primary).clicked() {
                    close = true;
                }
            });
        if close {
            self.data_notice = None;
        }
    }

    fn refresh_library_on_focus(&mut self, ctx: &egui::Context) {
        if self.scanning || self.dev.shot.is_some() || self.library.folders.is_empty() {
            return;
        }
        if let Some(last) = self.last_auto_scan {
            if last.elapsed() < Duration::from_secs(30) {
                return;
            }
        }
        self.last_auto_scan = Some(Instant::now());
        self.start_scan(None, ctx);
    }

    fn start_scan(&mut self, folder: Option<PathBuf>, ctx: &egui::Context) {
        if self.scanning {
            return;
        }
        self.save_library();
        let mut library = self.library.clone();
        let (tx, rx) = mpsc::channel();
        self.scan_rx = Some(rx);
        self.scanning = true;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let stats = if let Some(folder) = folder {
                library.scan_folder(&folder)
            } else {
                library.rescan()
            };
            let _ = tx.send((library, stats));
            ctx.request_repaint();
        });
    }

    fn poll_scan(&mut self, ctx: &egui::Context) {
        let Some(result) = self.scan_rx.as_ref().map(|rx| rx.try_recv()) else {
            return;
        };
        match result {
            Ok((library, stats)) => {
                self.library = library;
                self.library_textures.clear();
                self.scanning = false;
                self.scan_rx = None;
                self.migrate_progress();
                self.save_library();
                if stats.found == 0 {
                    self.say("No encontré EPUB en las carpetas vigiladas.");
                } else {
                    self.say(format!(
                        "{} encontrados · {} nuevos · {} actualizados · {} sin cambios · {} fallos",
                        stats.found, stats.added, stats.updated, stats.unchanged, stats.failed
                    ));
                }
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
            Err(TryRecvError::Disconnected) => {
                self.scanning = false;
                self.scan_rx = None;
                self.say("El escaneo se interrumpió.");
            }
        }
    }

    fn start_library_content_search(&mut self, ctx: &egui::Context) {
        if let Some(previous) = self.library_content.take() {
            previous.cancel.store(true, Ordering::Relaxed);
        }
        let query = self.library_search.trim().to_string();
        if query.chars().count() < CONTENT_SEARCH_MIN {
            self.say(format!(
                "Escribí al menos {CONTENT_SEARCH_MIN} caracteres para buscar en el contenido."
            ));
            return;
        }
        let available = self
            .library
            .books
            .iter()
            .filter(|book| !book.missing && book.error.is_none())
            .count();
        let books: Vec<(PathBuf, String)> = self
            .library
            .books
            .iter()
            .filter(|book| !book.missing && book.error.is_none())
            .take(CONTENT_SEARCH_MAX_BOOKS)
            .map(|book| (PathBuf::from(&book.path), book.title.clone()))
            .collect();
        if books.is_empty() {
            self.say("No hay libros disponibles para buscar en el contenido.");
            return;
        }

        let total = books.len();
        let truncated_books = available > CONTENT_SEARCH_MAX_BOOKS;
        let (tx, rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_thread = cancel.clone();
        let repaint = ctx.clone();
        let query_thread = query.clone();
        std::thread::spawn(move || {
            let mut scanned = 0usize;
            let mut failed = 0usize;
            let mut hits = 0usize;
            let mut truncated = truncated_books;
            for (path, title) in books {
                if cancel_thread.load(Ordering::Relaxed) {
                    break;
                }
                scanned += 1;
                match Book::open(&path) {
                    Ok(mut book) => {
                        let index = search::build(&mut book);
                        let remaining = CONTENT_SEARCH_MAX_HITS.saturating_sub(hits);
                        if remaining == 0 {
                            truncated = true;
                            break;
                        }
                        let per_book = remaining.min(CONTENT_SEARCH_PER_BOOK);
                        let (found, more) = search::search(&index, &query_thread, per_book);
                        if more {
                            truncated = true;
                        }
                        if !found.is_empty() {
                            let batch: Vec<LibraryContentHit> = found
                                .into_iter()
                                .map(|hit| {
                                    let chapter_title = index
                                        .chapters
                                        .get(hit.chapter)
                                        .map(|chapter| chapter.title.clone())
                                        .unwrap_or_default();
                                    LibraryContentHit {
                                        path: path.clone(),
                                        book_title: title.clone(),
                                        chapter: hit.chapter,
                                        chapter_title,
                                        block: hit.block,
                                        start: hit.start,
                                        end: hit.end,
                                        before: hit.before,
                                        matched: hit.matched,
                                        after: hit.after,
                                    }
                                })
                                .collect();
                            hits += batch.len();
                            let _ = tx.send(LibraryContentMsg::Hits(batch));
                        }
                        if hits >= CONTENT_SEARCH_MAX_HITS {
                            truncated = true;
                            let _ = tx.send(LibraryContentMsg::Progress {
                                scanned,
                                total,
                                failed,
                            });
                            break;
                        }
                    }
                    Err(_) => failed += 1,
                }
                let _ = tx.send(LibraryContentMsg::Progress {
                    scanned,
                    total,
                    failed,
                });
                repaint.request_repaint();
            }
            let _ = tx.send(LibraryContentMsg::Done {
                scanned,
                total,
                failed,
                truncated,
            });
            repaint.request_repaint();
        });

        self.library_content = Some(LibraryContentSearch {
            query,
            rx,
            cancel,
            hits: Vec::new(),
            scanned: 0,
            total,
            failed: 0,
            busy: true,
            truncated: truncated_books,
        });
        self.say(format!("Buscando en el contenido de {total} libros…"));
    }

    fn poll_library_content(&mut self, ctx: &egui::Context) {
        let Some(state) = &mut self.library_content else {
            return;
        };
        let mut done = false;
        loop {
            match state.rx.try_recv() {
                Ok(LibraryContentMsg::Progress {
                    scanned,
                    total,
                    failed,
                }) => {
                    state.scanned = scanned;
                    state.total = total;
                    state.failed = failed;
                }
                Ok(LibraryContentMsg::Hits(batch)) => state.hits.extend(batch),
                Ok(LibraryContentMsg::Done {
                    scanned,
                    total,
                    failed,
                    truncated,
                }) => {
                    state.scanned = scanned;
                    state.total = total;
                    state.failed = failed;
                    state.truncated = truncated;
                    state.busy = false;
                    done = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    state.busy = false;
                    break;
                }
            }
        }
        if state.busy {
            ctx.request_repaint_after(Duration::from_millis(120));
        } else if done {
            ctx.request_repaint();
        }
    }

    fn open_library_content_hit(&mut self, hit: LibraryContentHit, ctx: &egui::Context) {
        self.library_content = None;
        self.open(hit.path);
        if self.screen != Screen::Reader {
            return;
        }
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
        self.back_spot = None;
        self.say("Resultado de la búsqueda en el contenido.");
        ctx.request_repaint();
    }

    fn library_content_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(state) = &self.library_content else {
            return;
        };
        let query = state.query.clone();
        let hits = state.hits.clone();
        let scanned = state.scanned;
        let total = state.total;
        let failed = state.failed;
        let busy = state.busy;
        let truncated = state.truncated;
        let progress = if busy {
            Some(state.cancel.clone())
        } else {
            None
        };

        let mut open = true;
        let mut cancel = false;
        let mut jump: Option<LibraryContentHit> = None;
        egui::Window::new(
            RichText::new("Resultados en el contenido")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_MD)
        .max_width(widgets::PANEL_LG)
        .max_height(560.0)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
            ui.label(
                RichText::new(format!("«{query}» · {} resultados", hits.len()))
                    .size(theme::text::BODY_SMALL)
                    .color(th.fg),
            );
            if busy {
                ui.horizontal(|ui| {
                    ui.add(egui::Spinner::new());
                    ui.label(
                        RichText::new(format!("Buscando en {scanned} de {total} libros…"))
                            .size(theme::text::META)
                            .color(th.dim),
                    );
                    if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Ghost).clicked() {
                        cancel = true;
                    }
                });
            } else if hits.is_empty() {
                ui.label(
                    RichText::new("No se encontró el texto en los libros disponibles.")
                        .size(theme::text::META)
                        .color(th.dim),
                );
            }
            if failed > 0 {
                ui.label(
                    RichText::new(format!("{failed} libros no se pudieron abrir."))
                        .size(theme::text::META)
                        .color(th.dim),
                );
            }
            if truncated {
                ui.label(
                    RichText::new(format!(
                        "Lista parcial: máximo {CONTENT_SEARCH_MAX_HITS} resultados, {CONTENT_SEARCH_PER_BOOK} por libro y {CONTENT_SEARCH_MAX_BOOKS} libros."
                    ))
                    .size(theme::text::META)
                    .color(th.dim),
                );
            }
            ui.separator();
            ScrollArea::vertical().max_height(440.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                let body = FontId::new(theme::text::BODY_SMALL, fonts::family(false, false));
                let body_bold = FontId::new(theme::text::BODY_SMALL, fonts::family(true, false));
                let meta = FontId::new(theme::text::META, fonts::family(false, false));
                for (index, hit) in hits.iter().enumerate() {
                    let mut header = LayoutJob::default();
                    header.append(
                        &truncate(&hit.book_title, 42),
                        0.0,
                        TextFormat {
                            font_id: body_bold.clone(),
                            color: th.fg,
                            ..Default::default()
                        },
                    );
                    header.append(
                        &format!("   Cap. {}", hit.chapter + 1),
                        0.0,
                        TextFormat {
                            font_id: meta.clone(),
                            color: th.dim,
                            ..Default::default()
                        },
                    );
                    if !hit.chapter_title.trim().is_empty() {
                        header.append(
                            &format!("  ·  {}", truncate(&hit.chapter_title, 44)),
                            0.0,
                            TextFormat {
                                font_id: meta.clone(),
                                color: th.dim,
                                ..Default::default()
                            },
                        );
                    }
                    let mut snippet = LayoutJob::default();
                    snippet.append(
                        &hit.before,
                        0.0,
                        TextFormat {
                            font_id: body.clone(),
                            color: th.fg,
                            ..Default::default()
                        },
                    );
                    snippet.append(
                        &hit.matched,
                        0.0,
                        TextFormat {
                            font_id: body_bold.clone(),
                            color: th.accent,
                            ..Default::default()
                        },
                    );
                    snippet.append(
                        &hit.after,
                        0.0,
                        TextFormat {
                            font_id: body.clone(),
                            color: th.fg,
                            ..Default::default()
                        },
                    );

                    let card = egui::Frame::new()
                        .fill(th.bg)
                        .stroke(Stroke::new(1.0, th.border))
                        .corner_radius(theme::radius::SM)
                        .inner_margin(egui::Margin::symmetric(10, 8))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.add(egui::Label::new(header).wrap());
                            ui.add_space(4.0);
                            ui.add(egui::Label::new(snippet).wrap());
                        });
                    let rect = card.response.rect;
                    let id = ui.make_persistent_id(("content-hit", index));
                    let mut response = ui.interact(rect, id, Sense::click());
                    if response.has_focus() {
                        ui.painter().rect_stroke(
                            rect.expand(2.0),
                            theme::radius::SM + 1.0,
                            Stroke::new(2.0, th.accent),
                            egui::StrokeKind::Inside,
                        );
                    } else if response.hovered() {
                        ui.painter().rect_stroke(
                            rect.expand(1.0),
                            theme::radius::SM + 1.0,
                            Stroke::new(1.0, th.accent.gamma_multiply(0.55)),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let activated = response.clicked()
                        || (response.has_focus()
                            && ui.input(|i| {
                                i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)
                            }));
                    if activated {
                        jump = Some(hit.clone());
                    }
                    response = response
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text("Abrir en el libro");
                    let _ = response;
                    ui.add_space(6.0);
                }
            });
        });

        if cancel {
            if let Some(cancel) = progress {
                cancel.store(true, Ordering::Relaxed);
            }
            if let Some(state) = &mut self.library_content {
                state.busy = false;
            }
        }
        if !open {
            if let Some(state) = &self.library_content {
                if state.busy {
                    state.cancel.store(true, Ordering::Relaxed);
                }
            }
            self.library_content = None;
        }
        if let Some(hit) = jump {
            self.open_library_content_hit(hit, ctx);
        }
    }

    fn library_screen(&mut self, ctx: &egui::Context, th: &Theme) {
        let mut add_folder = false;
        let mut rescan = false;
        let mut open_path: Option<PathBuf> = None;
        let mut open_file = false;
        let mut remove: Option<usize> = None;
        let mut locate: Option<usize> = None;
        let mut recover: Option<usize> = None;
        let mut edit: Option<usize> = None;
        let mut remove_folder: Option<String> = None;
        let mut content_search = false;

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
        let folders = self.library.folders.clone();
        let has_books = !self.library.books.is_empty();
        let recent = self
            .library
            .books
            .iter()
            .filter(|book| !book.missing && book.error.is_none() && book.last_opened.is_some())
            .max_by_key(|book| book.last_opened.unwrap_or(0))
            .map(|book| (book.path.clone(), book.title.clone(), book.author.clone()));
        let show_second_row = has_books || !folders.is_empty();
        let wide = ctx.screen_rect().width() >= 820.0;
        let series_names = self.library.series_names();
        let has_no_series = self.library.books.iter().any(|book| {
            book.series
                .as_deref()
                .is_none_or(|name| name.trim().is_empty())
        });

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
                    ui.label(
                        RichText::new("Candil")
                            .size(theme::text::TITLE)
                            .strong()
                            .color(th.fg),
                    );
                    ui.add_space(2.0);
                    if self.scanning {
                        ui.add(egui::Spinner::new());
                        ui.label(
                            RichText::new("Escaneando…")
                                .size(theme::text::META)
                                .color(th.dim),
                        );
                    } else {
                        ui.label(
                            RichText::new(format!("{book_count} libros"))
                                .size(theme::text::META)
                                .color(th.dim),
                        );
                        if missing_count > 0 {
                            ui.label(
                                RichText::new(format!(
                                    "· {missing_count} falta{}",
                                    if missing_count == 1 { "" } else { "s" }
                                ))
                                .size(theme::text::META)
                                .color(th.dim),
                            );
                        }
                        if error_count > 0 {
                            let response = ui.label(
                                RichText::new(format!(
                                    "· {error_count} error{}",
                                    if error_count == 1 { "" } else { "es" }
                                ))
                                .size(theme::text::META)
                                .color(th.error),
                            );
                            if !error_details.is_empty() {
                                response.on_hover_text(error_details.clone());
                            }
                        }
                    }
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);

                    let compact = ui.available_width() < 520.0;
                    let show_continue = recent.is_some() && ui.available_width() >= 380.0;
                    let continue_reserve = if show_continue {
                        if compact {
                            92.0
                        } else {
                            158.0
                        }
                    } else {
                        0.0
                    };
                    let reserve = if compact { 130.0 } else { 208.0 } + continue_reserve;
                    let search_w = (ui.available_width() - reserve).max(150.0);
                    let search = egui::Frame::new()
                        .fill(th.bg)
                        .stroke(Stroke::new(1.0, th.border))
                        .corner_radius(theme::radius::MD)
                        .inner_margin(egui::Margin::symmetric(10, 5))
                        .show(ui, |ui| {
                            ui.set_min_height(32.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.library_search)
                                    .hint_text("Buscar título o autor… · Enter: texto")
                                    .desired_width((search_w - 20.0).max(120.0))
                                    .frame(false),
                            )
                        });
                    if search.inner.has_focus() {
                        ui.painter().rect_stroke(
                            search.response.rect,
                            theme::radius::MD,
                            Stroke::new(2.0, th.accent),
                            egui::StrokeKind::Inside,
                        );
                        if ctx.input(|i| i.key_pressed(Key::Enter)) {
                            content_search = true;
                        }
                    }
                    if search.inner.changed() {
                        if let Some(previous) = self.library_content.take() {
                            previous.cancel.store(true, Ordering::Relaxed);
                        }
                    }
                    if ctx.input(|i| {
                        (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::F)
                    }) {
                        search.inner.request_focus();
                    }
                    let _ = search
                        .response
                        .on_hover_text("Enter: buscar también dentro del contenido de los libros.");

                    if let Some((path, title, author)) = &recent {
                        if show_continue {
                            let label = if compact {
                                "Continuar"
                            } else {
                                "Continuar leyendo"
                            };
                            if widgets::button(ui, th, label, widgets::ButtonKind::Secondary)
                                .on_hover_text(format!("{title} · {author}"))
                                .clicked()
                            {
                                open_path = Some(PathBuf::from(path));
                            }
                        }
                    }

                    if !self.scanning
                        && widgets::button(
                            ui,
                            th,
                            if compact {
                                "Añadir"
                            } else {
                                "Añadir carpeta…"
                            },
                            widgets::ButtonKind::Primary,
                        )
                        .clicked()
                    {
                        add_folder = true;
                    }

                    ui.scope(|ui| {
                        let visuals = ui.visuals_mut();
                        visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                        visuals.widgets.hovered.weak_bg_fill = th.accent.gamma_multiply(0.12);
                        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                        visuals.widgets.active.bg_stroke = Stroke::new(2.0, th.accent);
                        ui.menu_button(
                            RichText::new("…").size(theme::text::TITLE).color(th.fg),
                            |ui| {
                                if widgets::button(
                                    ui,
                                    th,
                                    "Refrescar",
                                    widgets::ButtonKind::Secondary,
                                )
                                .clicked()
                                {
                                    rescan = true;
                                    ui.close();
                                }
                                if ui.button("Configuración").clicked() {
                                    self.show_config = true;
                                    self.config_tab = ConfigTab::Biblioteca;
                                    ui.close();
                                }
                                if ui.button("Atajos").clicked() {
                                    self.show_help = true;
                                    ui.close();
                                }
                                if ui.button("Abrir un EPUB…").clicked() {
                                    open_file = true;
                                    ui.close();
                                }
                                ui.separator();
                                ui.menu_button("Carpetas", |ui| {
                                    if folders.is_empty() {
                                        ui.label(
                                            RichText::new("No hay carpetas vigiladas.")
                                                .size(theme::text::META)
                                                .color(th.dim),
                                        );
                                    }
                                    for folder in &folders {
                                        let name = Path::new(folder)
                                            .file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| folder.clone());
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(truncate(&name, 28))
                                                    .size(theme::text::META)
                                                    .color(th.fg),
                                            )
                                            .on_hover_text(folder);
                                            if ui
                                                .small_button("×")
                                                .on_hover_text("Dejar de vigilar")
                                                .clicked()
                                            {
                                                remove_folder = Some(folder.clone());
                                                ui.close();
                                            }
                                        });
                                    }
                                    ui.separator();
                                    if widgets::button(
                                        ui,
                                        th,
                                        "Añadir carpeta…",
                                        widgets::ButtonKind::Primary,
                                    )
                                    .clicked()
                                    {
                                        add_folder = true;
                                        ui.close();
                                    }
                                });
                            },
                        )
                        .response
                        .on_hover_text("Más opciones");
                    });
                });
            });

        if content_search {
            self.start_library_content_search(ctx);
        }

        if show_second_row {
            egui::TopBottomPanel::top("library-filters")
                .show_separator_line(false)
                .exact_height(48.0)
                .frame(
                    egui::Frame::new()
                        .fill(th.bg)
                        .inner_margin(egui::Margin::symmetric(16, 8)),
                )
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        if !folders.is_empty() {
                            ui.label(
                                RichText::new("Carpetas")
                                    .size(theme::text::META)
                                    .color(th.dim),
                            );
                            for folder in folders.iter().take(4) {
                                let name = Path::new(folder)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| folder.clone());
                                let name = truncate(&name, if wide { 28 } else { 16 });
                                egui::Frame::new()
                                    .fill(th.panel)
                                    .stroke(Stroke::new(1.0, th.border))
                                    .corner_radius(theme::radius::SM)
                                    .inner_margin(egui::Margin::symmetric(8, 4))
                                    .show(ui, |ui| {
                                        ui.set_min_height(32.0);
                                        ui.horizontal_centered(|ui| {
                                            ui.label(
                                                RichText::new(name.clone())
                                                    .size(theme::text::META)
                                                    .color(th.fg),
                                            )
                                            .on_hover_text(folder);
                                            if ui
                                                .small_button(
                                                    RichText::new("×").size(theme::text::META),
                                                )
                                                .on_hover_text("Dejar de vigilar")
                                                .clicked()
                                            {
                                                remove_folder = Some(folder.clone());
                                            }
                                        });
                                    });
                            }
                            if folders.len() > 4 {
                                ui.label(
                                    RichText::new(format!("+{}", folders.len() - 4))
                                        .size(theme::text::META)
                                        .color(th.dim),
                                );
                            }
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);
                            ui.spacing_mut().interact_size.y = 32.0;
                            egui::ComboBox::from_id_salt("library-filter")
                                .width(if wide { 160.0 } else { 136.0 })
                                .selected_text(
                                    RichText::new(self.library_filter.label())
                                        .size(theme::text::BODY_SMALL)
                                        .color(th.fg),
                                )
                                .show_ui(ui, |ui| {
                                    let mut filters = vec![
                                        LibraryFilter::All,
                                        LibraryFilter::Reading,
                                        LibraryFilter::Unread,
                                        LibraryFilter::Missing,
                                        LibraryFilter::Duplicates,
                                    ];
                                    for name in &series_names {
                                        filters.push(LibraryFilter::Series(name.clone()));
                                    }
                                    if has_no_series {
                                        filters.push(LibraryFilter::NoSeries);
                                    }
                                    for filter in &filters {
                                        ui.selectable_value(
                                            &mut self.library_filter,
                                            filter.clone(),
                                            filter.label(),
                                        );
                                    }
                                });
                            if wide {
                                ui.label(
                                    RichText::new("Filtro")
                                        .size(theme::text::META)
                                        .color(th.dim),
                                );
                                ui.add_space(4.0);
                            }
                            egui::ComboBox::from_id_salt("library-order")
                                .width(if wide { 124.0 } else { 104.0 })
                                .selected_text(
                                    RichText::new(self.library_order.label())
                                        .size(theme::text::BODY_SMALL)
                                        .color(th.fg),
                                )
                                .show_ui(ui, |ui| {
                                    for order in [
                                        LibraryOrder::Recent,
                                        LibraryOrder::Title,
                                        LibraryOrder::Author,
                                        LibraryOrder::Series,
                                        LibraryOrder::Added,
                                    ] {
                                        ui.selectable_value(
                                            &mut self.library_order,
                                            order,
                                            order.label(),
                                        );
                                    }
                                });
                            if wide {
                                ui.label(
                                    RichText::new("Orden").size(theme::text::META).color(th.dim),
                                );
                            }
                        });
                    });
                });
        }

        let indices = self.visible_books();
        let empty = self.library.books.is_empty();
        let has_folders = !folders.is_empty();
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
                if empty && self.scanning {
                    ui.vertical_centered(|ui| {
                        ui.add_space(120.0);
                        ui.add(egui::Spinner::new());
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Escaneando la biblioteca…")
                                .size(theme::text::BODY)
                                .color(th.dim),
                        );
                    });
                    return;
                }

                if empty {
                    ui.vertical_centered(|ui| {
                        ui.add_space(90.0);
                        ui.label(
                            RichText::new(empty_title)
                                .size(theme::text::DISPLAY)
                                .color(th.fg),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(empty_subtitle)
                                .size(theme::text::BODY)
                                .color(th.dim),
                        );
                        ui.add_space(22.0);
                        if widgets::button(ui, th, "Añadir carpeta…", widgets::ButtonKind::Primary)
                            .clicked()
                        {
                            add_folder = true;
                        }
                        if has_folders {
                            ui.add_space(8.0);
                            if widgets::button(ui, th, "Refrescar", widgets::ButtonKind::Ghost)
                                .clicked()
                            {
                                rescan = true;
                            }
                        }
                        ui.add_space(8.0);
                        if widgets::button(ui, th, "Abrir un EPUB…", widgets::ButtonKind::Secondary)
                            .clicked()
                        {
                            open_file = true;
                        }
                        ui.add_space(18.0);
                        ui.label(
                            RichText::new("o arrastrá un archivo .epub a esta ventana")
                                .size(theme::text::BODY_SMALL)
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
                        ui.label(
                            RichText::new(message)
                                .size(theme::text::HEADLINE)
                                .color(th.fg),
                        );
                        ui.add_space(10.0);
                        let clear = if self.library_search.trim().is_empty() {
                            "Ver todos"
                        } else {
                            "Limpiar búsqueda"
                        };
                        if widgets::button(ui, th, clear, widgets::ButtonKind::Secondary).clicked()
                        {
                            if self.library_search.trim().is_empty() {
                                self.library_filter = LibraryFilter::All;
                            } else {
                                self.library_search.clear();
                            }
                        }
                        if self.library_search.trim().chars().count() >= CONTENT_SEARCH_MIN {
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(
                                    "También podés buscar dentro del texto de los libros.",
                                )
                                .size(theme::text::META)
                                .color(th.dim),
                            );
                            if widgets::button(
                                ui,
                                th,
                                "Buscar en el contenido",
                                widgets::ButtonKind::Ghost,
                            )
                            .clicked()
                            {
                                self.start_library_content_search(ctx);
                            }
                        }
                    });
                    return;
                }

                let virtualize = indices.len() >= GRID_VIRTUALIZE_AFTER;
                let series_by_pos: Vec<Option<String>> = if virtualize {
                    indices
                        .iter()
                        .map(|&index| {
                            self.library
                                .books
                                .get(index)
                                .and_then(|book| book.series.as_deref())
                                .map(str::trim)
                                .filter(|name| !name.is_empty())
                                .map(str::to_string)
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let grouped = virtualize
                    && self.library_order == LibraryOrder::Series
                    && series_by_pos.iter().any(Option::is_some);
                let scroll = egui::ScrollArea::vertical()
                    .id_salt("library-grid")
                    .auto_shrink([false, false]);
                let scroll = if let Some(offset) = self.dev.library_scroll {
                    scroll.vertical_scroll_offset(offset)
                } else {
                    scroll
                };

                if virtualize {
                    scroll.show_viewport(ui, |ui, viewport| {
                        let avail = ui.available_width();
                        let cols = ((avail - PAGE_PAD * 2.0 + CARD_GAP) / (CARD_W + CARD_GAP))
                            .floor()
                            .max(1.0) as usize;
                        let row_w = cols as f32 * CARD_W + cols.saturating_sub(1) as f32 * CARD_GAP;
                        let left = ((avail - row_w) * 0.5).max(PAGE_PAD);
                        let items = build_grid_items(&series_by_pos, cols, grouped);
                        let total_h = GRID_TOP_PAD
                            + items.iter().map(GridItem::height).sum::<f32>()
                            + GRID_BOTTOM_PAD;
                        ui.set_height(total_h);
                        let top = ui.max_rect().top();
                        let mut y = GRID_TOP_PAD;

                        for item in &items {
                            let height = item.height();
                            let rect = egui::Rect::from_min_size(
                                egui::pos2(ui.max_rect().left(), top + y),
                                Vec2::new(ui.max_rect().width(), height),
                            );
                            let visible = y + height >= viewport.min.y - GRID_ROW_GAP
                                && y <= viewport.max.y + GRID_ROW_GAP;
                            if visible {
                                match item {
                                    GridItem::Header { label, count } => {
                                        ui.scope_builder(
                                            egui::UiBuilder::new().max_rect(rect),
                                            |ui| {
                                                ui.add_space(6.0);
                                                ui.horizontal(|ui| {
                                                    ui.add_space(left);
                                                    ui.label(
                                                        RichText::new(truncate(label, 60))
                                                            .size(theme::text::LABEL)
                                                            .strong()
                                                            .color(th.fg),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "· {} libro{}",
                                                            count,
                                                            if *count == 1 { "" } else { "s" }
                                                        ))
                                                        .size(theme::text::META)
                                                        .color(th.dim),
                                                    );
                                                });
                                            },
                                        );
                                    }
                                    GridItem::Row { start, end } => {
                                        let cards = self.cards(&indices[*start..*end], ctx);
                                        ui.scope_builder(
                                            egui::UiBuilder::new().max_rect(rect),
                                            |ui| {
                                                ui.horizontal_top(|ui| {
                                                    ui.add_space(left);
                                                    for card in &cards {
                                                        let focus =
                                                            self.dev.focus_card && *start == 0;
                                                        match library_card(ui, card, th, focus) {
                                                            Some(CardAction::Open) => {
                                                                open_path =
                                                                    Some(PathBuf::from(&card.path));
                                                            }
                                                            Some(CardAction::Remove) => {
                                                                remove = Some(card.index)
                                                            }
                                                            Some(CardAction::Locate) => {
                                                                locate = Some(card.index)
                                                            }
                                                            Some(CardAction::Recover) => {
                                                                recover = Some(card.index)
                                                            }
                                                            Some(CardAction::Edit) => {
                                                                edit = Some(card.index)
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                });
                                            },
                                        );
                                    }
                                }
                            }
                            y += height;
                        }
                    });
                } else {
                    let cards = self.cards(&indices, ctx);
                    scroll.show(ui, |ui| {
                        ui.add_space(GRID_TOP_PAD);
                        let avail = ui.available_width();
                        let cols = ((avail - PAGE_PAD * 2.0 + CARD_GAP) / (CARD_W + CARD_GAP))
                            .floor()
                            .max(1.0) as usize;
                        let row_w = cols as f32 * CARD_W + cols.saturating_sub(1) as f32 * CARD_GAP;
                        let left = ((avail - row_w) * 0.5).max(PAGE_PAD);
                        ui.spacing_mut().item_spacing.x = CARD_GAP;

                        let mut card_pos = 0usize;
                        let grouped = self.library_order == LibraryOrder::Series
                            && cards.iter().any(|card| card.series_name.is_some());
                        let mut groups: Vec<(Option<String>, Vec<&CardData>)> = Vec::new();
                        if grouped {
                            for card in &cards {
                                match groups.last_mut() {
                                    Some((name, group))
                                        if matches!(
                                            (name.as_deref(), card.series_name.as_deref()),
                                            (Some(a), Some(b)) if a.eq_ignore_ascii_case(b)
                                        ) =>
                                    {
                                        group.push(card);
                                    }
                                    _ => groups.push((card.series_name.clone(), vec![card])),
                                }
                            }
                        } else {
                            groups.push((None, cards.iter().collect()));
                        }

                        for (series, group) in &groups {
                            if grouped {
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(left);
                                    ui.label(
                                        RichText::new(series.as_deref().unwrap_or("Sin serie"))
                                            .size(theme::text::LABEL)
                                            .strong()
                                            .color(th.fg),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "· {} libro{}",
                                            group.len(),
                                            if group.len() == 1 { "" } else { "s" }
                                        ))
                                        .size(theme::text::META)
                                        .color(th.dim),
                                    );
                                });
                                ui.add_space(10.0);
                            }
                            for row in group.chunks(cols) {
                                ui.horizontal_top(|ui| {
                                    ui.add_space(left);
                                    for card in row {
                                        let focus = self.dev.focus_card && card_pos == 0;
                                        card_pos += 1;
                                        match library_card(ui, card, th, focus) {
                                            Some(CardAction::Open) => {
                                                open_path = Some(PathBuf::from(&card.path));
                                            }
                                            Some(CardAction::Remove) => remove = Some(card.index),
                                            Some(CardAction::Locate) => locate = Some(card.index),
                                            Some(CardAction::Recover) => recover = Some(card.index),
                                            Some(CardAction::Edit) => edit = Some(card.index),
                                            None => {}
                                        }
                                    }
                                });
                                ui.add_space(GRID_ROW_GAP);
                            }
                        }
                        ui.add_space(GRID_BOTTOM_PAD);
                    });
                }
            });

        if add_folder {
            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                self.start_scan(Some(dir), ctx);
            }
        }
        if rescan && !self.scanning {
            self.start_scan(None, ctx);
        }
        if open_file {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("EPUB", &["epub"])
                .pick_file()
            {
                open_path = Some(p);
            }
        }
        if let Some(folder) = remove_folder {
            self.confirm_remove_folder = Some(folder);
        }
        if let Some(index) = remove {
            self.confirm_remove_book = Some(index);
        }
        if let Some(index) = locate {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("EPUB", &["epub"])
                .pick_file()
            {
                self.relink_book(index, path);
            }
        }
        if let Some(index) = recover {
            self.confirm_recover_book = Some(index);
        }
        if let Some(index) = edit {
            self.open_epub_editor(index, ctx);
        }
        if let Some(path) = open_path {
            self.open(path);
        }

        self.remove_book_window(ctx, th);
        self.remove_folder_window(ctx, th);
        self.recover_book_window(ctx, th);
        self.epub_editor_window(ctx, th);
    }

    fn open_epub_editor(&mut self, index: usize, ctx: &egui::Context) {
        let Some(entry) = self.library.books.get(index) else {
            return;
        };
        if entry.missing || entry.error.is_some() {
            self.say("No puedo editar un EPUB faltante o con error.");
            return;
        }
        let path = PathBuf::from(&entry.path);
        match epub_edit::read_info(&path) {
            Ok(info) => {
                let cover_ext = info
                    .cover
                    .as_ref()
                    .map(|cover| CoverExt::from_hint(&cover.media_type, &cover.href))
                    .unwrap_or(CoverExt::Jpg);
                let cover_texture = info
                    .cover_bytes
                    .as_deref()
                    .and_then(|bytes| texture_from_image(ctx, bytes));
                let fallback_title = path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_default();
                self.epub_editor = Some(EpubEditor {
                    index,
                    title: if info.title.trim().is_empty() {
                        fallback_title
                    } else {
                        info.title
                    },
                    author: info.author,
                    lang: info.lang,
                    path,
                    cover_ext,
                    cover_upload: None,
                    cover_texture,
                    error: None,
                });
            }
            Err(error) => self.say(format!("No pude leer el EPUB: {error}")),
        }
    }

    fn epub_editor_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if self.epub_editor.is_none() {
            return;
        }

        let mut save_request: Option<(usize, PathBuf, Edits)> = None;
        let mut cancel = false;
        {
            let editor = self.epub_editor.as_mut().expect("editor comprobado");
            let file_name = editor
                .path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let mut pick_cover = false;
            let mut save = false;
            let mut close = false;

            let _ = egui::Modal::new(egui::Id::new("epub-editor-modal"))
                .backdrop_color(Color32::from_black_alpha(40))
                .frame(widgets::panel_frame(th))
                .show(ctx, |ui| {
                    ui.set_width(520.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Editar EPUB")
                                .size(theme::text::TITLE)
                                .strong()
                                .color(th.fg),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                                .on_hover_text("Cancelar")
                                .clicked()
                            {
                                close = true;
                            }
                        });
                    });
                    ui.separator();
                    widgets::meta(ui, th, file_name.clone());
                    ui.add_space(theme::space::SM);

                    widgets::label(ui, th, "Título");
                    ui.add(
                        egui::TextEdit::singleline(&mut editor.title)
                            .desired_width(f32::INFINITY),
                    );
                    widgets::label(ui, th, "Autor");
                    ui.add(
                        egui::TextEdit::singleline(&mut editor.author)
                            .desired_width(f32::INFINITY),
                    );
                    widgets::label(ui, th, "Idioma");
                    ui.add(egui::TextEdit::singleline(&mut editor.lang).desired_width(120.0));

                    ui.add_space(theme::space::SM);
                    ui.horizontal_top(|ui| {
                        if let Some(texture) = &editor.cover_texture {
                            let size = texture.size_vec2();
                            let scale = (160.0 / size.x).min(220.0 / size.y).min(1.0);
                            ui.add(egui::Image::new((texture.id(), size * scale)));
                        } else {
                            ui.add_sized(
                                [160.0, 120.0],
                                egui::Label::new(
                                    RichText::new("Sin portada")
                                        .size(theme::text::META)
                                        .color(th.dim),
                                ),
                            );
                        }
                        ui.vertical(|ui| {
                            widgets::label(ui, th, "Portada");
                            if widgets::button(
                                ui,
                                th,
                                "Elegir imagen…",
                                widgets::ButtonKind::Secondary,
                            )
                            .clicked()
                            {
                                pick_cover = true;
                            }
                            widgets::meta(ui, th, "JPG, PNG o WebP; se ajusta a 1600×2400");
                        });
                    });

                    ui.add_space(theme::space::SM);
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                "Se reescribe el EPUB. Antes se guarda un backup en el directorio de datos.",
                            )
                            .size(theme::text::META)
                            .color(th.dim),
                        )
                        .wrap(),
                    );
                    if let Some(error) = &editor.error {
                        ui.add(
                            egui::Label::new(
                                RichText::new(error)
                                    .size(theme::text::META)
                                    .color(th.error),
                            )
                            .wrap(),
                        );
                    }
                    ui.add_space(theme::space::SM);
                    ui.horizontal(|ui| {
                        if widgets::button(
                            ui,
                            th,
                            "Guardar cambios",
                            widgets::ButtonKind::Primary,
                        )
                        .clicked()
                        {
                            save = true;
                        }
                        if widgets::button(
                            ui,
                            th,
                            "Cancelar",
                            widgets::ButtonKind::Secondary,
                        )
                        .clicked()
                        {
                            close = true;
                        }
                    });
                });

            if pick_cover {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Imagen", &["jpg", "jpeg", "png", "webp"])
                    .pick_file()
                {
                    match std::fs::read(&path)
                        .map_err(|e| format!("No pude leer la imagen: {e}"))
                        .and_then(|bytes| prepare_cover(&bytes, editor.cover_ext))
                    {
                        Ok(bytes) => {
                            editor.cover_texture = texture_from_image(ctx, &bytes);
                            editor.cover_upload = Some(bytes);
                            editor.error = None;
                        }
                        Err(error) => editor.error = Some(error),
                    }
                }
            }

            if save {
                if editor.title.trim().is_empty() {
                    editor.error = Some("El título no puede quedar vacío.".into());
                } else {
                    save_request = Some((
                        editor.index,
                        editor.path.clone(),
                        Edits {
                            title: editor.title.trim().to_string(),
                            author: editor.author.trim().to_string(),
                            lang: editor.lang.trim().to_string(),
                            cover: editor.cover_upload.clone().map(|bytes| CoverUpload {
                                bytes,
                                ext: editor.cover_ext,
                            }),
                        },
                    ));
                }
            }
            if close {
                cancel = true;
            }
        }

        if let Some((index, path, edits)) = save_request {
            let backup_dir = self.library.data_dir.join("backups");
            match epub_edit::save(&path, &edits, &backup_dir) {
                Ok(result) => {
                    self.epub_editor = None;
                    self.apply_epub_edit(index, &result.backup);
                }
                Err(error) => {
                    if let Some(editor) = self.epub_editor.as_mut() {
                        editor.error = Some(error);
                    }
                }
            }
        } else if cancel {
            self.epub_editor = None;
        }
    }

    fn apply_epub_edit(&mut self, index: usize, backup: &Path) {
        let Some(old) = self.library.books.get(index).cloned() else {
            return;
        };
        let mut entry =
            BookEntry::from_path_with_cover(&old.path, Some(&self.library.data_dir.join("covers")));
        entry.progress = old.progress;
        entry.settings = old.settings;
        entry.added = old.added;
        entry.last_opened = old.last_opened;
        self.library.books[index] = entry;
        self.library_textures.clear();
        self.save_library();
        self.say(format!("EPUB actualizado. Backup: {}", backup.display()));
    }

    fn remove_book_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(index) = self.confirm_remove_book else {
            return;
        };
        let Some(title) = self.library.books.get(index).map(|b| b.title.clone()) else {
            self.confirm_remove_book = None;
            return;
        };
        let mut confirm = false;
        let mut cancel = false;
        let _ = egui::Modal::new(egui::Id::new("remove-book-modal"))
            .backdrop_color(Color32::from_black_alpha(40))
            .frame(widgets::panel_frame(th))
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Quitar de la biblioteca")
                            .size(theme::text::TITLE)
                            .strong()
                            .color(th.fg),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                            .on_hover_text("Cancelar")
                            .clicked()
                        {
                            cancel = true;
                        }
                    });
                });
                ui.separator();
                ui.label(
                    RichText::new(format!("¿Quitar «{title}» de la biblioteca?"))
                        .size(theme::text::BODY)
                        .color(th.fg),
                );
                ui.add_space(theme::space::SM);
                ui.label(
                    RichText::new("El archivo EPUB no se borra; solo se quita del índice.")
                        .size(theme::text::META)
                        .color(th.dim),
                );
                ui.add_space(theme::space::MD);
                ui.horizontal(|ui| {
                    if widgets::button(ui, th, "Quitar", widgets::ButtonKind::Destructive).clicked()
                    {
                        confirm = true;
                    }
                    if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if confirm {
            if index < self.library.books.len() {
                self.library.books.remove(index);
                self.save_library();
                self.say("Libro quitado de la biblioteca.");
            }
            self.confirm_remove_book = None;
        } else if cancel {
            self.confirm_remove_book = None;
        }
    }

    fn remove_folder_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(folder) = self.confirm_remove_folder.clone() else {
            return;
        };
        let name = Path::new(&folder)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| folder.clone());
        let mut confirm = false;
        let mut cancel = false;
        let _ = egui::Modal::new(egui::Id::new("remove-folder-modal"))
            .backdrop_color(Color32::from_black_alpha(40))
            .frame(widgets::panel_frame(th))
            .show(ctx, |ui| {
                ui.set_width(400.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Dejar de vigilar carpeta")
                            .size(theme::text::TITLE)
                            .strong()
                            .color(th.fg),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                            .on_hover_text("Cancelar")
                            .clicked()
                        {
                            cancel = true;
                        }
                    });
                });
                ui.separator();
                ui.label(
                    RichText::new(format!("¿Dejar de vigilar «{name}»?"))
                        .size(theme::text::BODY)
                        .color(th.fg),
                );
                ui.add_space(theme::space::SM);
                ui.label(
                    RichText::new(
                        "Los libros ya indexados siguen en la biblioteca; no se borra ningún archivo.",
                    )
                    .size(theme::text::META)
                    .color(th.dim),
                );
                ui.add_space(theme::space::MD);
                ui.horizontal(|ui| {
                    if widgets::button(
                        ui,
                        th,
                        "Dejar de vigilar",
                        widgets::ButtonKind::Destructive,
                    )
                    .clicked()
                    {
                        confirm = true;
                    }
                    if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if confirm {
            self.library.unwatch_folder(&folder);
            self.save_library();
            self.confirm_remove_folder = None;
            self.say(format!("Dejé de vigilar {name}."));
        } else if cancel {
            self.confirm_remove_folder = None;
        }
    }

    fn recover_book_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(index) = self.confirm_recover_book else {
            return;
        };
        let Some(book) = self.library.books.get(index) else {
            self.confirm_recover_book = None;
            return;
        };
        let title = book.title.clone();
        let error = book.error.clone();
        let missing = book.missing;
        let path = book.path.clone();
        let file_label = Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        let mut locate = false;
        let mut remove = false;
        let mut cancel = false;

        let _ = egui::Modal::new(egui::Id::new("recover-book-modal"))
            .backdrop_color(Color32::from_black_alpha(40))
            .frame(widgets::panel_frame(th))
            .show(ctx, |ui| {
                ui.set_width(440.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if missing {
                            "Archivo faltante"
                        } else {
                            "No se pudo abrir el EPUB"
                        })
                        .size(theme::text::TITLE)
                        .strong()
                        .color(th.fg),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                            .on_hover_text("Cancelar")
                            .clicked()
                        {
                            cancel = true;
                        }
                    });
                });
                ui.separator();
                ui.label(
                    RichText::new(format!("«{title}»"))
                        .size(theme::text::BODY)
                        .strong()
                        .color(th.fg),
                );
                ui.add_space(theme::space::SM);
                if missing {
                    ui.label(
                        RichText::new("El archivo ya no está en la ruta indexada.")
                            .size(theme::text::BODY)
                            .color(th.dim),
                    );
                } else if let Some(error) = &error {
                    ui.add(
                        egui::Label::new(
                            RichText::new(error).size(theme::text::BODY).color(th.error),
                        )
                        .wrap(),
                    );
                }
                ui.add_space(theme::space::XS);
                ui.add(
                    egui::Label::new(
                        RichText::new(file_label)
                            .size(theme::text::META)
                            .color(th.dim),
                    )
                    .wrap(),
                )
                .on_hover_text(path);
                ui.add_space(theme::space::MD);
                ui.horizontal_wrapped(|ui| {
                    if widgets::button(ui, th, "Ubicar archivo…", widgets::ButtonKind::Primary)
                        .clicked()
                    {
                        locate = true;
                    }
                    if widgets::button(
                        ui,
                        th,
                        "Quitar de la biblioteca",
                        widgets::ButtonKind::Destructive,
                    )
                    .clicked()
                    {
                        remove = true;
                    }
                    if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if locate {
            self.confirm_recover_book = None;
            if let Some(new_path) = rfd::FileDialog::new()
                .add_filter("EPUB", &["epub"])
                .pick_file()
            {
                self.relink_book(index, new_path);
            }
        } else if remove {
            self.confirm_recover_book = None;
            self.confirm_remove_book = Some(index);
        } else if cancel {
            self.confirm_recover_book = None;
        }
    }

    fn visible_books(&self) -> Vec<usize> {
        let query = collate(self.library_search.trim());
        let duplicates = duplicate_counts(&self.library.books);
        let mut out: Vec<usize> = self
            .library
            .books
            .iter()
            .enumerate()
            .filter(|(_, b)| matches_filter(&self.library_filter, b, &duplicates))
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
            LibraryOrder::Series => out.sort_by(|&a, &b| {
                let ba = &self.library.books[a];
                let bb = &self.library.books[b];
                let sa = ba.series.as_deref().unwrap_or("");
                let sb = bb.series.as_deref().unwrap_or("");
                sa.is_empty()
                    .cmp(&sb.is_empty())
                    .then_with(|| collate(sa).cmp(&collate(sb)))
                    .then_with(|| {
                        ba.series_index
                            .unwrap_or(f32::INFINITY)
                            .partial_cmp(&bb.series_index.unwrap_or(f32::INFINITY))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
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
        let duplicates = duplicate_counts(&self.library.books);
        indices
            .iter()
            .filter_map(|&index| {
                let entry = self.library.books.get(index)?.clone();
                let cover = entry
                    .cover
                    .as_deref()
                    .and_then(|rel| cover_texture(ctx, &mut self.library_textures, &data_dir, rel));
                let series_name = entry
                    .series
                    .as_deref()
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_string);
                let duplicate_count = duplicate_key(&entry)
                    .and_then(|key| duplicates.get(&key).copied())
                    .filter(|count| *count > 1)
                    .unwrap_or(0);
                Some(CardData {
                    index,
                    path: entry.path,
                    title: entry.title,
                    author: entry.author,
                    series: series_name.as_ref().map(|name| match entry.series_index {
                        Some(index) if index.fract().abs() < 0.01 => {
                            format!("{name} · {}", index.round() as i64)
                        }
                        Some(index) => format!("{name} · {index:.1}"),
                        None => name.clone(),
                    }),
                    series_name,
                    duplicate_count,
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
        let compact = ctx.screen_rect().width() < 820.0;
        let mut back = false;

        egui::TopBottomPanel::top("bar")
            .show_separator_line(false)
            .exact_height(40.0)
            .frame(
                egui::Frame::new()
                    .fill(th.panel)
                    .inner_margin(egui::Margin::symmetric(16, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if widgets::button(ui, th, "←", widgets::ButtonKind::Ghost)
                        .on_hover_text("Biblioteca (Esc)")
                        .clicked()
                    {
                        back = true;
                    }
                    if widgets::button(ui, th, "☰", widgets::ButtonKind::Ghost)
                        .on_hover_text("Índice (T)")
                        .clicked()
                    {
                        self.show_toc = !self.show_toc;
                        self.show_bookmarks = false;
                        if self.show_toc {
                            self.show_goto = false;
                            self.show_search = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
                    }
                    if widgets::button(
                        ui,
                        th,
                        if book_override { "Aa·" } else { "Aa" },
                        widgets::ButtonKind::Ghost,
                    )
                    .on_hover_text(if book_override {
                        "Tipografía (S) · este libro tiene ajustes propios"
                    } else {
                        "Tipografía (S)"
                    })
                    .clicked()
                    {
                        self.show_settings = !self.show_settings;
                        self.show_bookmarks = false;
                        if self.show_settings {
                            self.show_goto = false;
                            self.show_search = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
                    }
                    if widgets::button(ui, th, "Notas", widgets::ButtonKind::Ghost)
                        .on_hover_text("Destacados y notas (D)")
                        .clicked()
                    {
                        self.show_annotations = !self.show_annotations;
                        self.show_toc = false;
                        self.show_settings = false;
                        self.show_bookmarks = false;
                        if self.show_annotations {
                            self.show_goto = false;
                            self.show_search = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
                    }
                    if widgets::button(ui, th, "Buscar", widgets::ButtonKind::Ghost)
                        .on_hover_text("Buscar en el libro (⌘F)")
                        .clicked()
                    {
                        self.show_search = true;
                        self.search_focus = true;
                        self.show_toc = false;
                        self.show_settings = false;
                        self.show_annotations = false;
                        self.show_bookmarks = false;
                        self.show_goto = false;
                        self.selection = None;
                        self.note = None;
                    }
                    if widgets::button(
                        ui,
                        th,
                        if compact { "Marca" } else { "Marcador" },
                        widgets::ButtonKind::Ghost,
                    )
                    .on_hover_text("Marcadores (B guarda · Mayús+B abre)")
                    .clicked()
                    {
                        self.show_bookmarks = !self.show_bookmarks;
                        self.show_toc = false;
                        self.show_settings = false;
                        self.show_annotations = false;
                        self.show_search = false;
                        self.show_goto = false;
                        self.search_highlight = None;
                        self.pending_search_jump = None;
                        self.selection = None;
                    }

                    ui.add_space(theme::space::SM);
                    let shown = if chapter_title.is_empty() {
                        title.clone()
                    } else {
                        format!("{} · {}", title, chapter_title)
                    };
                    let author_reserve = if !compact && !author.is_empty() {
                        170.0
                    } else {
                        0.0
                    };
                    let reserve = if compact {
                        220.0
                    } else {
                        240.0 + author_reserve
                    };
                    let title_w = (ui.available_width() - reserve).max(110.0);
                    ui.add_sized(
                        [title_w, 20.0],
                        egui::Label::new(
                            RichText::new(truncate(&shown, 120))
                                .size(theme::text::BODY)
                                .color(th.fg),
                        )
                        .truncate(),
                    );
                    if !compact && !author.is_empty() && ui.available_width() > 90.0 {
                        ui.label(
                            RichText::new(truncate(&author, 34))
                                .size(theme::text::LABEL)
                                .color(th.dim),
                        );
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact {
                            ui.scope(|ui| {
                                let visuals = ui.visuals_mut();
                                visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                                visuals.widgets.inactive.bg_stroke = Stroke::NONE;
                                visuals.widgets.hovered.weak_bg_fill =
                                    th.accent.gamma_multiply(0.12);
                                visuals.widgets.hovered.bg_stroke = Stroke::NONE;
                                visuals.widgets.active.bg_stroke = Stroke::new(2.0, th.accent);
                                ui.menu_button(
                                    RichText::new("…").size(theme::text::TITLE).color(th.fg),
                                    |ui| {
                                        if ui.button("Configuración").clicked() {
                                            self.open_config(ctx, ConfigTab::Consultas);
                                            ui.close();
                                        }
                                        if ui.button("Marcadores").clicked() {
                                            self.show_bookmarks = true;
                                            self.show_toc = false;
                                            self.show_settings = false;
                                            self.show_annotations = false;
                                            self.show_search = false;
                                            self.show_goto = false;
                                            ui.close();
                                        }
                                        if ui.button("Atajos").clicked() {
                                            self.show_help = true;
                                            ui.close();
                                        }
                                    },
                                )
                                .response
                                .on_hover_text("Más opciones");
                            });
                        } else if widgets::button(ui, th, "Ajustes", widgets::ButtonKind::Ghost)
                            .on_hover_text("Configuración (⌘,)")
                            .clicked()
                        {
                            self.open_config(ctx, ConfigTab::Consultas);
                        }

                        let pct = ui.add_sized(
                            [44.0, 32.0],
                            egui::Label::new(
                                RichText::new(format!("{:.0}%", self.progress() * 100.0))
                                    .size(theme::text::BODY_SMALL)
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
                            self.show_bookmarks = false;
                            self.search_highlight = None;
                            self.pending_search_jump = None;
                        }
                        if widgets::button(ui, th, "›", widgets::ButtonKind::Ghost)
                            .on_hover_text("Capítulo siguiente (→)")
                            .clicked()
                        {
                            self.goto_chapter(1);
                        }
                        ui.label(
                            RichText::new(format!("{} / {}", self.chapter + 1, n))
                                .size(theme::text::BODY_SMALL)
                                .color(th.dim),
                        );
                        if widgets::button(ui, th, "‹", widgets::ButtonKind::Ghost)
                            .on_hover_text("Capítulo anterior (←)")
                            .clicked()
                        {
                            self.goto_chapter(-1);
                        }
                    });
                });
            });
        if back {
            self.go_library();
        }
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
        let toc_h = (ctx.screen_rect().height() - 120.0).clamp(320.0, 540.0);
        egui::Window::new(
            RichText::new("Índice")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .min_width(widgets::PANEL_MD)
        .max_width(widgets::PANEL_MD)
        .default_width(widgets::PANEL_MD)
        .default_height(toc_h)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = theme::space::XS;
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
                    let text = RichText::new(title)
                        .size(theme::text::BODY)
                        .color(if selected { th.accent } else { th.fg });
                    let fill = if selected {
                        Color32::from_rgba_unmultiplied(
                            th.accent.r(),
                            th.accent.g(),
                            th.accent.b(),
                            38,
                        )
                    } else {
                        Color32::TRANSPARENT
                    };
                    let clicked = ui
                        .horizontal(|ui| {
                            if selected {
                                let (rect, _) =
                                    ui.allocate_exact_size(Vec2::new(3.0, 18.0), Sense::hover());
                                ui.painter().rect_filled(rect, 1.0, th.accent);
                            } else {
                                ui.add_space(3.0);
                            }
                            ui.add_space(theme::space::XS);
                            egui::Frame::new()
                                .fill(fill)
                                .corner_radius(theme::radius::SM)
                                .inner_margin(egui::Margin::symmetric(6, 3))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.add(egui::Label::new(text).wrap().sense(Sense::click()))
                                        .clicked()
                                })
                                .inner
                        })
                        .inner;
                    if clicked {
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

    fn bookmarks_window(&mut self, ctx: &egui::Context, th: &Theme) {
        if !self.show_bookmarks {
            return;
        }
        let Some(book) = self.book.as_mut() else {
            self.show_bookmarks = false;
            return;
        };
        let n = book.chapter_count();
        let titles: Vec<String> = (0..n).map(|index| book.chapter_title(index)).collect();
        let Some(key) = self.book_key() else {
            self.show_bookmarks = false;
            return;
        };
        let bookmarks: Vec<Bookmark> = self
            .library
            .books
            .iter()
            .find(|entry| entry.path == key)
            .map(|entry| entry.bookmarks.clone())
            .unwrap_or_default();

        let mut open = self.show_bookmarks;
        let mut add_here = false;
        let mut jump: Option<Progress> = None;
        let mut remove: Option<u64> = None;

        egui::Window::new(
            RichText::new("Marcadores")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_SM)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = theme::space::XS;
            if widgets::button(
                ui,
                th,
                "Añadir marcador aquí",
                widgets::ButtonKind::Secondary,
            )
            .on_hover_text("También podés usar B en el lector")
            .clicked()
            {
                add_here = true;
            }
            ui.label(
                RichText::new("B guarda la posición actual; Mayús+B abre esta lista.")
                    .size(theme::text::META)
                    .color(th.dim),
            );
            ui.separator();
            if bookmarks.is_empty() {
                ui.label(
                    RichText::new("Todavía no hay marcadores en este libro.")
                        .size(theme::text::META)
                        .color(th.dim),
                );
                return;
            }
            ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                for bookmark in &bookmarks {
                    let title = if bookmark.label.trim().is_empty() {
                        titles
                            .get(bookmark.progress.chapter)
                            .filter(|title| !title.trim().is_empty())
                            .cloned()
                            .unwrap_or_else(|| {
                                format!("Capítulo {}", bookmark.progress.chapter + 1)
                            })
                    } else {
                        bookmark.label.clone()
                    };
                    let fraction = ((bookmark.progress.chapter as f32 + bookmark.progress.ratio)
                        / n.max(1) as f32)
                        .clamp(0.0, 1.0);
                    ui.horizontal(|ui| {
                        let percent = (fraction * 100.0).round() as u32;
                        let label = format!("{} · {percent}%", truncate(&title, 30));
                        let response = ui.add(
                            egui::Button::new(
                                RichText::new(label)
                                    .size(theme::text::BODY_SMALL)
                                    .color(th.fg),
                            )
                            .frame(false),
                        );
                        if response.clicked() {
                            jump = Some(bookmark.progress);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button(RichText::new("×").size(theme::text::META))
                                .on_hover_text("Eliminar marcador")
                                .clicked()
                            {
                                remove = Some(bookmark.id);
                            }
                        });
                    });
                }
            });
        });
        self.show_bookmarks = open;
        if add_here {
            self.add_bookmark_here();
        }
        if let Some(id) = remove {
            self.remove_bookmark(id);
        }
        if let Some(progress) = jump {
            self.jump_to_bookmark(progress);
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

        egui::Window::new(
            RichText::new("Tipografía")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_SM)
        .frame(widgets::panel_frame(th))
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
                            .size(theme::text::META)
                            .color(th.dim),
                    );
                }
            } else if has_book {
                ui.label(
                    RichText::new("El libro no está en la biblioteca; los ajustes son globales.")
                        .size(theme::text::META)
                        .color(th.dim),
                );
            }

            widgets::label(ui, th, "Tema");
            ui.horizontal_wrapped(|ui| {
                for (i, t) in THEMES.iter().enumerate() {
                    if widgets::theme_swatch(ui, th, t, i == settings.theme)
                        .on_hover_text(t.name)
                        .clicked()
                    {
                        settings.theme = i;
                        changed = true;
                    }
                }
            });

            ui.add_space(4.0);
            changed |= ui
                .add(
                    egui::Slider::new(&mut settings.font_size, config::FONT_MIN..=config::FONT_MAX)
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
                if widgets::button(ui, th, "Restablecer", widgets::ButtonKind::Secondary).clicked()
                {
                    let theme_keep = settings.theme;
                    settings = Settings::default();
                    settings.theme = theme_keep;
                    changed = true;
                }
                widgets::meta(ui, th, "Space / ← → / T / S");
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
            (screen.right() - 400.0).max(screen.left() + 8.0),
        );

        let mut highlight = false;
        let mut note = false;
        let mut consultar = false;
        egui::Area::new(egui::Id::new("selection_bar"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(egui::Stroke::new(1.0, th.border))
                    .corner_radius(theme::radius::MD)
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (index, name) in ["acento", "ámbar", "verde"].iter().enumerate() {
                                let color = highlight_swatch(th, index as u8);
                                if widgets::color_swatch(
                                    ui,
                                    th,
                                    color,
                                    self.active_color == index as u8,
                                    &format!("Color {name}"),
                                )
                                .clicked()
                                {
                                    self.active_color = index as u8;
                                }
                            }
                            ui.separator();
                            if widgets::button(ui, th, "Destacar", widgets::ButtonKind::Secondary)
                                .on_hover_text(
                                    "Destacar selección (H) · Mayúsculas + clic extiende entre párrafos",
                                )
                                .clicked()
                            {
                                highlight = true;
                            }
                            if widgets::button(ui, th, "Nota", widgets::ButtonKind::Ghost)
                                .on_hover_text("Destacar con nota (N)")
                                .clicked()
                            {
                                note = true;
                            }
                            if widgets::button(ui, th, "Consultar", widgets::ButtonKind::Ghost)
                                .on_hover_text(
                                    "Diccionario, traducción o IA (K) · Mayúsculas + K traduce directo",
                                )
                                .clicked()
                            {
                                consultar = true;
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
        if consultar {
            self.open_consulta(ctx);
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

        let query = self.notes_query.clone();
        let filter_rows = |rows: Vec<AnnRow>| -> Vec<AnnRow> {
            if query.trim().is_empty() {
                return rows;
            }
            rows.into_iter()
                .filter(|row| {
                    note_matches(
                        &query,
                        &row.annotation.quote,
                        &row.annotation.note,
                        &row.book_title,
                        &row.chapter_title,
                    )
                })
                .collect()
        };
        let rows_book = filter_rows(rows_book);
        let rows_all = filter_rows(rows_all);
        let filtering = !self.notes_query.trim().is_empty();

        let mut open = self.show_annotations;
        let mut remove = None;
        let mut edit = None;
        let mut jump = None;
        let mut recolor = None;
        let mut export = false;

        egui::Window::new(
            RichText::new("Destacados y notas")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_MD)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.annotations_global, false, "Este libro");
                ui.selectable_value(&mut self.annotations_global, true, "Todos");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if widgets::button(ui, th, "Exportar…", widgets::ButtonKind::Secondary)
                        .on_hover_text("Exportar las citas visibles a Markdown")
                        .clicked()
                    {
                        export = true;
                    }
                });
            });
            ui.horizontal(|ui| {
                let width = (ui.available_width() - 30.0).max(120.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.notes_query)
                        .hint_text("Filtrar citas y notas…")
                        .desired_width(width),
                );
                if !self.notes_query.is_empty()
                    && ui
                        .small_button("×")
                        .on_hover_text("Limpiar filtro")
                        .clicked()
                {
                    self.notes_query.clear();
                }
            });
            ui.separator();

            let rows = if self.annotations_global {
                &rows_all
            } else {
                &rows_book
            };

            if rows.is_empty() {
                let message = if filtering {
                    "No hay citas ni notas que coincidan."
                } else if self.annotations_global {
                    "Todavía no hay destacados en ningún libro."
                } else {
                    "Todavía no destacaste nada."
                };
                ui.label(
                    RichText::new(message)
                        .size(theme::text::BODY_SMALL)
                        .color(th.dim),
                );
                ui.add_space(4.0);
                if filtering {
                    if widgets::button(ui, th, "Limpiar filtro", widgets::ButtonKind::Secondary)
                        .clicked()
                    {
                        self.notes_query.clear();
                    }
                } else {
                    ui.label(
                        RichText::new("Seleccioná una frase y usá Destacar o Nota.")
                            .size(theme::text::META)
                            .color(th.dim),
                    );
                }
                return;
            }

            egui::ScrollArea::vertical()
                .max_height(430.0)
                .show(ui, |ui| {
                    for row in rows {
                        let a = &row.annotation;
                        ui.horizontal_top(|ui| {
                            let color_button = widgets::color_swatch(
                                ui,
                                th,
                                highlight_swatch(th, a.color),
                                false,
                                "Cambiar color",
                            );
                            if color_button.clicked() {
                                recolor = Some((a.id, (a.color + 1) % 3));
                            }
                            ui.label(
                                RichText::new(format!(
                                    "“{}”",
                                    truncate(&quote_inline(&a.quote), 150)
                                ))
                                .size(theme::text::BODY)
                                .color(th.fg),
                            );
                        });
                        if row.missing {
                            ui.label(
                                RichText::new("(no encontrada en este capítulo)")
                                    .size(theme::text::META)
                                    .color(th.error),
                            );
                        }
                        if !a.note.is_empty() {
                            ui.label(
                                RichText::new(truncate(&a.note, 220))
                                    .size(theme::text::LABEL)
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
                            let place = if a.paragraph_count() > 1 {
                                format!("{place} · {} párrafos", a.paragraph_count())
                            } else {
                                place
                            };
                            ui.label(RichText::new(place).size(theme::text::META).color(th.dim));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                                    .on_hover_text("Borrar destacado")
                                    .clicked()
                                {
                                    remove = Some(a.id);
                                }
                                if widgets::button(ui, th, "Editar", widgets::ButtonKind::Ghost)
                                    .on_hover_text("Editar la nota y el color")
                                    .clicked()
                                {
                                    edit = Some(a.id);
                                }
                                if widgets::button(
                                    ui,
                                    th,
                                    "Ir al texto",
                                    widgets::ButtonKind::Ghost,
                                )
                                .clicked()
                                {
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
            self.confirm_delete_annotation = Some(id);
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

    fn confirm_delete_annotation_window(&mut self, ctx: &egui::Context, th: &Theme) {
        let Some(id) = self.confirm_delete_annotation else {
            return;
        };
        let Some(annotation) = self.annotations.get(id).cloned() else {
            self.confirm_delete_annotation = None;
            return;
        };
        let mut confirm = false;
        let mut cancel = false;
        let _ = egui::Modal::new(egui::Id::new("delete-annotation-modal"))
            .backdrop_color(Color32::from_black_alpha(40))
            .frame(widgets::panel_frame(th))
            .show(ctx, |ui| {
                ui.set_width(380.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Borrar destacado")
                            .size(theme::text::TITLE)
                            .strong()
                            .color(th.fg),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if widgets::button(ui, th, "×", widgets::ButtonKind::Ghost)
                            .on_hover_text("Cancelar")
                            .clicked()
                        {
                            cancel = true;
                        }
                    });
                });
                ui.separator();
                ui.add(
                    egui::Label::new(
                        RichText::new(format!(
                            "“{}”",
                            truncate(&quote_inline(&annotation.quote), 160)
                        ))
                        .size(theme::text::BODY)
                        .color(th.fg),
                    )
                    .wrap(),
                );
                ui.add_space(theme::space::SM);
                ui.label(
                    RichText::new("Se borra el destacado y su nota; el texto del libro no cambia.")
                        .size(theme::text::META)
                        .color(th.dim),
                );
                ui.add_space(theme::space::MD);
                ui.horizontal(|ui| {
                    if widgets::button(ui, th, "Borrar", widgets::ButtonKind::Destructive).clicked()
                    {
                        confirm = true;
                    }
                    if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if confirm {
            self.confirm_delete_annotation = None;
            self.remove_annotation(id);
        } else if cancel {
            self.confirm_delete_annotation = None;
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

        egui::Window::new(
            RichText::new(title)
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_MD)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.horizontal(|ui| {
                widgets::label(ui, th, "Color");
                for (index, name) in ["acento", "ámbar", "verde"].iter().enumerate() {
                    let color = highlight_swatch(th, index as u8);
                    if widgets::color_swatch(
                        ui,
                        th,
                        color,
                        editor.color == index as u8,
                        &format!("Color {name}"),
                    )
                    .clicked()
                    {
                        editor.color = index as u8;
                    }
                }
            });
            widgets::label(ui, th, "Nota (opcional; vacía deja solo el destacado)");
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
                if widgets::button(ui, th, "Guardar", widgets::ButtonKind::Primary).clicked() {
                    save = true;
                }
                if widgets::button(ui, th, "Cancelar", widgets::ButtonKind::Secondary).clicked() {
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

        egui::Window::new(
            RichText::new("Ir a")
                .size(theme::text::TITLE)
                .strong()
                .color(th.fg),
        )
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(widgets::PANEL_SM)
        .frame(widgets::panel_frame(th))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            widgets::label(ui, th, "Porcentaje del libro");
            ui.add(
                egui::Slider::new(&mut self.goto_percent, 0.0..=100.0)
                    .suffix(" %")
                    .step_by(0.5),
            );
            if widgets::button(ui, th, "Ir al porcentaje", widgets::ButtonKind::Primary).clicked() {
                by_percent = true;
            }

            ui.add_space(theme::space::SM);
            ui.separator();
            ui.add_space(theme::space::XS);
            widgets::label(ui, th, format!("Capítulo (1–{chapters})"));
            ui.add(
                egui::DragValue::new(&mut self.goto_chapter_number)
                    .range(1..=chapters)
                    .speed(0.2),
            );
            if widgets::button(ui, th, "Ir al capítulo", widgets::ButtonKind::Secondary).clicked()
            {
                by_chapter = true;
            }
            ui.add_space(theme::space::XS);
            widgets::meta(ui, th, "G para abrir/cerrar · Esc para cerrar");
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
                    .stroke(Stroke::new(1.0, th.border))
                    .corner_radius(theme::radius::MD)
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        ui.label(RichText::new(msg).size(theme::text::BODY).color(th.fg));
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(400));
    }

    // -- lectura -------------------------------------------------------------

    fn reader(&mut self, ctx: &egui::Context, th: &Theme) {
        self.top_bar(ctx, th);
        if self.screen == Screen::Library {
            return;
        }

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
        if let Some(selection) = self.selection.as_ref() {
            if selection.chapter == self.chapter {
                let color = highlight_color(th, self.active_color.min(2));
                for segment in selection_segments(selection, &self.blocks) {
                    highlights.entry(segment.block).or_default().push((
                        segment.start,
                        segment.end,
                        color,
                    ));
                }
            }
        }
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
        let mut link_click: Option<LinkClick> = None;
        let mut external_link: Option<String> = None;

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
                let mut extend_to: Option<Anchor> = None;
                let mut geoms: Vec<BlockGeom> = Vec::new();

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
                                        blocks.as_slice(),
                                        current_chapter,
                                        &mut link_click,
                                        &mut external_link,
                                        selection,
                                        &mut extend_to,
                                        &mut geoms,
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

                // Extiende la selección cuando el puntero sale del bloque ancla.
                // Mayúsculas + clic ya dejó su destino en `extend_to`.
                let pointer_pos = ctx.input(|i| i.pointer.interact_pos());
                let primary_down = ctx.input(|i| i.pointer.primary_down());
                if let Some(current) = selection.clone() {
                    if current.chapter == current_chapter {
                        if let (Some(start), Some(end)) = (
                            anchor_screen_rect(&geoms, current.anchor),
                            anchor_screen_rect(&geoms, current.focus),
                        ) {
                            if let Some(selection) = selection.as_mut() {
                                selection.rect = start.union(end);
                            }
                        }
                        let target = extend_to.take().or_else(|| {
                            if !primary_down {
                                return None;
                            }
                            let pos = pointer_pos?;
                            let anchor_rect = geoms
                                .iter()
                                .find(|geom| geom.block == current.anchor.block)
                                .map(|geom| geom.rect)?;
                            if pos.y >= anchor_rect.top() - 2.0
                                && pos.y <= anchor_rect.bottom() + 2.0
                            {
                                return None;
                            }
                            pointer_anchor(&geoms, pos, pos.y < anchor_rect.top())
                        });
                        if let Some(target) = target {
                            let rect = match (
                                anchor_screen_rect(&geoms, current.anchor),
                                anchor_screen_rect(&geoms, target),
                            ) {
                                (Some(start), Some(end)) => start.union(end),
                                _ => current.rect,
                            };
                            if let Some(new) = selection_from_anchors(
                                current_chapter,
                                blocks.as_slice(),
                                current.anchor,
                                target,
                                rect,
                                current.edit_id,
                            ) {
                                *selection = Some(new);
                                ctx.request_repaint();
                            }
                        }
                    }
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
            if !self.dev.selection {
                self.selection = None;
                self.reader_focus = None;
            }
            if !self.dev.link_preview {
                self.link_preview = None;
            }
        }

        if let Some(click) = link_click {
            self.open_link_preview(click);
            ctx.request_repaint();
        }
        if let Some(url) = external_link {
            self.external_link = Some(url);
            ctx.request_repaint();
        }

        // Evita que el layout de la barra se desplace cuando cambia el ancho.
        let _ = colw;
        self.toc_window(ctx, th);
        self.settings_window(ctx, th);
        self.search_window(ctx, th);
        self.goto_window(ctx, th);
        self.selection_bar(ctx, th);
        self.link_preview_window(ctx, th);
        self.external_link_window(ctx, th);
        self.back_to_text(ctx, th);
        self.annotations_window(ctx, th);
        self.bookmarks_window(ctx, th);
        self.confirm_delete_annotation_window(ctx, th);
        self.consulta_ui(ctx, th);
        self.note_window(ctx, th);
    }

    fn screenshot(&mut self, ctx: &egui::Context) {
        if self.dev.shot.is_none() {
            return;
        }
        self.shot_frames += 1;

        let content_busy = self
            .library_content
            .as_ref()
            .is_some_and(|state| state.busy);
        if self.shot_frames >= 12 && !self.consulta_busy && !content_busy && !self.shot_requested {
            self.shot_requested = true;
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
/// si el bloque o el texto cambiaron, busca las partes de la cita en orden.
fn resolve_annotation(annotation: &Annotation, blocks: &[Block]) -> Option<Vec<Segment>> {
    if !annotation.is_multi_block() {
        if let Some(block) = blocks.get(annotation.block) {
            let text = block.text();
            if char_slice_eq(&text, annotation.start, annotation.end, &annotation.quote) {
                return Some(vec![Segment {
                    block: annotation.block,
                    start: annotation.start,
                    end: annotation.end,
                }]);
            }
            if let Some((start, end)) = find_quote(&text, &annotation.quote) {
                return Some(vec![Segment {
                    block: annotation.block,
                    start,
                    end,
                }]);
            }
        }
        return blocks.iter().enumerate().find_map(|(index, block)| {
            find_quote(&block.text(), &annotation.quote).map(|(start, end)| {
                vec![Segment {
                    block: index,
                    start,
                    end,
                }]
            })
        });
    }

    let parts: Vec<&str> = if annotation.parts.is_empty() {
        annotation.quote.split('\n').collect()
    } else {
        annotation.parts.iter().map(String::as_str).collect()
    };
    let parts: Vec<&str> = parts
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect();
    if parts.is_empty() {
        return None;
    }

    if let Some(segments) = exact_segments(annotation, blocks, &parts) {
        return Some(segments);
    }

    let mut cursor = annotation.block.min(blocks.len());
    let mut segments = Vec::with_capacity(parts.len());
    for part in parts {
        let mut found = None;
        for (index, block) in blocks.iter().enumerate().skip(cursor) {
            if let Some((start, end)) = find_quote(&block.text(), part) {
                found = Some(Segment {
                    block: index,
                    start,
                    end,
                });
                break;
            }
        }
        let segment = found?;
        cursor = segment.block + 1;
        segments.push(segment);
    }
    (!segments.is_empty()).then_some(segments)
}

/// Verifica los offsets exactos de un rango multi-bloque contra sus partes.
fn exact_segments(
    annotation: &Annotation,
    blocks: &[Block],
    parts: &[&str],
) -> Option<Vec<Segment>> {
    let start_block = annotation.block;
    let end_block = annotation.end_block();
    if start_block > end_block || end_block >= blocks.len() {
        return None;
    }
    let mut parts = parts.iter();
    let mut segments = Vec::new();
    for (block, current) in blocks
        .iter()
        .enumerate()
        .take(end_block + 1)
        .skip(start_block)
    {
        let text = current.text();
        let len = text.chars().count();
        let from = if block == start_block {
            annotation.start.min(len)
        } else {
            0
        };
        let to = if block == end_block {
            annotation.end.min(len)
        } else {
            len
        };
        if to <= from {
            continue;
        }
        let expected = parts.next()?;
        let actual: String = text.chars().skip(from).take(to - from).collect();
        if actual != *expected {
            return None;
        }
        segments.push(Segment {
            block,
            start: from,
            end: to,
        });
    }
    if parts.next().is_some() {
        return None;
    }
    (!segments.is_empty()).then_some(segments)
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
        if let Some(segments) = resolve_annotation(annotation, blocks) {
            let color = highlight_color(theme, annotation.color);
            for segment in segments {
                out.entry(segment.block)
                    .or_default()
                    .push((segment.start, segment.end, color));
            }
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

fn mix_white(color: Color32, amount: f32) -> Color32 {
    let mix =
        |value: u8| (value as f32 + (255.0 - value as f32) * amount.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgb(mix(color.r()), mix(color.g()), mix(color.b()))
}

/// La consulta flota sobre el libro: un tono más claro que el panel y una
/// sombra suave para que se lea como una capa aparte.
fn consulta_window_fill(theme: &Theme) -> Color32 {
    mix_white(theme.panel, if theme.is_dark() { 0.14 } else { 0.80 })
}

fn resolve_history_range(
    blocks: &[Block],
    block: usize,
    quote: &str,
) -> Option<(usize, usize, usize)> {
    if let Some(found) = find_history_quote(blocks, block, quote) {
        return Some(found);
    }
    // Una consulta multi-bloque guarda el texto unido con espacios. Si la
    // cita completa no cabe en un bloque, se prueba el prefijo más largo
    // formado por palabras completas.
    let words: Vec<&str> = quote.split_whitespace().collect();
    for take in (1..words.len()).rev() {
        let candidate = words[..take].join(" ");
        if let Some(found) = find_history_quote(blocks, block, &candidate) {
            return Some(found);
        }
    }
    None
}

fn find_history_quote(
    blocks: &[Block],
    block: usize,
    quote: &str,
) -> Option<(usize, usize, usize)> {
    let index = locate_quote(blocks, block, quote)?;
    let text = blocks[index].text();
    let (start, end) = find_quote(&text, quote)?;
    Some((index, start, end))
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
            Screen::Library => "Candil — biblioteca".to_string(),
            Screen::Reader => self
                .book
                .as_ref()
                .map(|book| format!("Candil — {}", book.title))
                .unwrap_or_else(|| "Candil".to_string()),
        };
        if self.window_title != title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }

        if self.dev.rescan {
            self.dev.rescan = false;
            self.start_scan(None, ctx);
        }
        self.handle_input(ctx);
        let focused = ctx.input(|i| i.focused);
        if self.screen == Screen::Library && focused && !self.was_focused {
            self.refresh_library_on_focus(ctx);
        }
        self.was_focused = focused;
        self.poll_scan(ctx);
        self.poll_library_content(ctx);
        self.poll_consulta();
        self.poll_update(ctx);
        if self.consulta_busy {
            ctx.request_repaint_after(Duration::from_millis(120));
        }

        match self.screen {
            Screen::Library => self.library_screen(ctx, &th),
            Screen::Reader => self.reader(ctx, &th),
        }

        self.config_window(ctx, &th);
        self.library_content_window(ctx, &th);
        self.data_notice_window(ctx, &th);
        self.help_window(ctx, &th);
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

fn context_around(text: &str, start: usize, end: usize, max: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max {
        return text.trim().to_string();
    }
    let start = start.min(chars.len());
    let end = end.min(chars.len()).max(start);
    let half = max.saturating_sub(end - start) / 2;
    let from = start.saturating_sub(half);
    let to = (end + half).min(chars.len());
    let mut out: String = chars[from..to].iter().collect();
    if from > 0 {
        out.insert(0, '…');
    }
    if to < chars.len() {
        out.push('…');
    }
    out
}

fn demo_response(kind: ConsultaKind) -> ConsultaResponse {
    let (provider, model, sections) = match kind {
        ConsultaKind::Diccionario => (
            "Wiktionary".to_string(),
            None,
            vec![
                consulta::Section {
                    title: "Noun".into(),
                    body: "The state of reduced consciousness during which the body rests.".into(),
                },
                consulta::Section {
                    title: "Verb".into(),
                    body: "To rest in a state of reduced consciousness.\nEjemplo: You should sleep eight hours a day.".into(),
                },
            ],
        ),
        ConsultaKind::Traduccion => (
            "MyMemory".to_string(),
            None,
            vec![consulta::Section {
                title: "EN → ES".into(),
                body: "el sueño de la razón".into(),
            }],
        ),
        ConsultaKind::Ia => (
            "OpenCode Go".to_string(),
            Some("deepseek-v4.1-flash".into()),
            vec![consulta::Section {
                title: "Explicar en contexto".into(),
                body: "El pasaje presenta la razón como un límite que se vuelve contra sí mismo cuando pretende explicarlo todo. La imagen del sueño sugiere que la claridad también tiene un revés irracional.".into(),
            }],
        ),
    };
    ConsultaResponse {
        kind,
        provider,
        model,
        source_url: None,
        sections,
        raw: String::new(),
    }
}

fn new_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("candil-{}-{nanos:x}", std::process::id())
}

fn open_path(path: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

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

fn texture_from_image(ctx: &egui::Context, bytes: &[u8]) -> Option<egui::TextureHandle> {
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, image.as_raw());
    Some(ctx.load_texture("epub-editor-cover", color, egui::TextureOptions::LINEAR))
}

fn prepare_cover(bytes: &[u8], ext: CoverExt) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("No pude leer la imagen: {e}"))?;
    let (width, height) = (img.width(), img.height());
    if width == 0 || height == 0 {
        return Err("La imagen no tiene tamaño.".into());
    }
    let scale = (1600.0 / width as f32).min(2400.0 / height as f32).min(1.0);
    let img = if scale < 1.0 {
        img.resize(
            (width as f32 * scale).round().max(1.0) as u32,
            (height as f32 * scale).round().max(1.0) as u32,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img
    };
    let mut out = Vec::new();
    match ext {
        CoverExt::Png => {
            img.write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
                .map_err(|e| format!("No pude convertir la portada: {e}"))?;
        }
        CoverExt::Jpg => {
            let rgb = flatten_white(img);
            let mut cursor = Cursor::new(&mut out);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 90);
            encoder
                .encode_image(&image::DynamicImage::ImageRgb8(rgb))
                .map_err(|e| format!("No pude convertir la portada: {e}"))?;
        }
    }
    Ok(out)
}

fn flatten_white(img: image::DynamicImage) -> image::RgbImage {
    let rgba = img.to_rgba8();
    let mut out = image::RgbImage::new(rgba.width(), rgba.height());
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel[3] as u32;
        let blend = |channel: u8| ((channel as u32 * alpha + 255 * (255 - alpha)) / 255) as u8;
        out.put_pixel(
            x,
            y,
            image::Rgb([blend(pixel[0]), blend(pixel[1]), blend(pixel[2])]),
        );
    }
    out
}

fn human_age(created: i64) -> String {
    let secs = crate::library::now().saturating_sub(created).max(0);
    if secs < 60 {
        "recién".into()
    } else if secs < 3_600 {
        format!("hace {} min", secs / 60)
    } else if secs < 86_400 {
        format!("hace {} h", secs / 3_600)
    } else {
        format!("hace {} d", secs / 86_400)
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Cita en una sola línea para el panel: colapsa saltos y espacios repetidos.
fn quote_inline(quote: &str) -> String {
    quote.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Filtro del panel de anotaciones: cita, nota, libro o capítulo.
fn note_matches(query: &str, quote: &str, note: &str, book: &str, chapter: &str) -> bool {
    let query = collate(query.trim());
    if query.is_empty() {
        return true;
    }
    collate(quote).contains(&query)
        || collate(note).contains(&query)
        || collate(book).contains(&query)
        || collate(chapter).contains(&query)
}

/// ¿El texto del enlace parece un marcador de nota (`1`, `iv`, `[2]`)?
fn looks_like_note_marker(label: &str) -> bool {
    let label = label.trim();
    if label.is_empty() || label.chars().count() > 6 {
        return false;
    }
    label.chars().all(|ch| {
        ch.is_ascii_digit()
            || matches!(
                ch,
                'i' | 'v' | 'x' | 'I' | 'V' | 'X' | '[' | ']' | '*' | '.' | '-'
            )
    })
}

/// Texto de la nota o del bloque destino, para la vista previa.
fn preview_text(blocks: &[Block], anchor: Option<&str>, max_chars: usize) -> String {
    let start = anchor
        .and_then(|anchor| {
            blocks
                .iter()
                .position(|block| block.anchor.as_deref() == Some(anchor))
        })
        .or_else(|| {
            blocks
                .iter()
                .position(|block| !block.text().trim().is_empty())
        })
        .unwrap_or(0);
    let mut texts = blocks.iter().skip(start).filter_map(|block| {
        let text = block.text().trim().to_string();
        (!text.is_empty()).then_some((block.kind, text))
    });
    let Some(first) = texts.next() else {
        return String::new();
    };
    let text = if matches!(first.0, Kind::Heading(_)) {
        texts.next().map(|(_, text)| text).unwrap_or(first.1)
    } else {
        first.1
    };
    truncate(&text, max_chars)
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

#[cfg(test)]
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

#[allow(clippy::too_many_arguments)]
fn add_selectable_text(
    ui: &mut Ui,
    job: LayoutJob,
    args: &TextArgs<'_>,
    blocks: &[Block],
    link_click: &mut Option<LinkClick>,
    external_link: &mut Option<String>,
    selection: &mut Option<Selection>,
    extend_to: &mut Option<Anchor>,
    geoms: &mut Vec<BlockGeom>,
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

    if !args.block_text.is_empty() {
        geoms.push(BlockGeom {
            block: args.block_index,
            rect: egui::Rect::from_min_size(output.galley_pos, output.galley.size()),
            galley: output.galley.clone(),
            prefix_len: args.prefix_len,
            text_len: args.block_text.chars().count(),
        });
    }

    // Mayúsculas + clic: extiende desde el ancla actual hasta el bloque
    // clickeado. Se procesa al soltar y no reemplaza la selección existente.
    let shift = ui.input(|i| i.modifiers.shift);
    let shift_extend = response.clicked()
        && shift
        && selection.is_some()
        && ui.input(|i| i.pointer.any_released());
    if shift_extend {
        if let Some(pos) = response.interact_pointer_pos() {
            let offset = output
                .galley
                .cursor_from_pos(pos - output.galley_pos)
                .index
                .saturating_sub(args.prefix_len)
                .min(args.block_text.chars().count());
            *extend_to = Some(Anchor {
                block: args.block_index,
                offset,
            });
        }
    }

    let pointer_down = ui.input(|i| i.pointer.primary_down());
    if !shift_extend {
        if let Some(range) = output.cursor_range {
            let primary = range.primary.index;
            let secondary = range.secondary.index;
            let focused = response.has_focus();
            let collapsed = primary == secondary;
            if focused && collapsed {
                // No borrar una selección multi-bloque al soltar ni durante el
                // arrastre; sí al hacer un clic normal o al colapsar una
                // selección de un solo bloque.
                if !pointer_down
                    && !shift
                    && (response.clicked()
                        || !selection.as_ref().is_some_and(Selection::is_multi_block))
                {
                    *selection = None;
                }
            } else if focused {
                let keep_multi = selection.as_ref().is_some_and(Selection::is_multi_block)
                    && !pointer_down
                    && !response.clicked();
                if !keep_multi {
                    let anchor = Anchor {
                        block: args.block_index,
                        offset: secondary.saturating_sub(args.prefix_len),
                    };
                    let focus = Anchor {
                        block: args.block_index,
                        offset: primary.saturating_sub(args.prefix_len),
                    };
                    let start_rect = output
                        .galley
                        .pos_from_cursor(egui::text::CCursor::new(primary.min(secondary)));
                    let end_rect = output
                        .galley
                        .pos_from_cursor(egui::text::CCursor::new(primary.max(secondary)));
                    let rect = egui::Rect::from_two_pos(
                        output.galley_pos + start_rect.left_top().to_vec2(),
                        output.galley_pos + end_rect.right_bottom().to_vec2(),
                    );
                    if let Some(new) = selection_from_anchors(
                        args.chapter,
                        blocks,
                        anchor,
                        focus,
                        rect,
                        response.id,
                    ) {
                        *selection = Some(new);
                    } else if !pointer_down {
                        *selection = None;
                    }
                }
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

    if !shift_extend && !args.links.is_empty() && response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let index = output.galley.cursor_from_pos(pos - output.galley_pos).index;
            if let Some((range, href)) = args.links.iter().find(|(range, _)| range.contains(&index))
            {
                if let Some(url) = Book::external_url(href) {
                    *external_link = Some(url);
                } else if let Some(book) = args.book {
                    if let Some((spine, anchor)) = book.link_target(href, args.chapter) {
                        let label = args
                            .full_text
                            .chars()
                            .skip(range.start)
                            .take(range.end.saturating_sub(range.start))
                            .collect::<String>();
                        *link_click = Some(LinkClick {
                            spine,
                            anchor,
                            rect: egui::Rect::from_min_size(pos, egui::vec2(1.0, 1.0)),
                            label,
                        });
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

fn reveal_path(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let dir = path.parent().unwrap_or(path);
        let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
    }
}

fn open_external(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

fn library_card(ui: &mut Ui, card: &CardData, th: &Theme, focus: bool) -> Option<CardAction> {
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
            if card.duplicate_count > 0 {
                let text = format!("Posible duplicado ×{}", card.duplicate_count);
                let font = FontId::new(theme::text::META, fonts::family(true, false));
                let galley = ui.painter().layout_no_wrap(text, font, th.bg);
                let pad = Vec2::new(6.0, 3.0);
                let size = galley.size() + pad * 2.0;
                let rect = egui::Rect::from_min_size(cover_rect.min + Vec2::new(8.0, 8.0), size);
                ui.painter()
                    .rect_filled(rect, 4.0, th.accent.gamma_multiply(0.92));
                ui.painter().galley(rect.min + pad, galley, th.bg);
            }

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
                                .size(theme::text::BODY_SMALL)
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
                                    .size(theme::text::META)
                                    .color(th.dim),
                            )
                            .wrap(),
                        );
                    }
                    if let Some(series) = &card.series {
                        ui.add(
                            egui::Label::new(
                                RichText::new(truncate(series, 34))
                                    .size(theme::text::META)
                                    .italics()
                                    .color(th.dim.gamma_multiply(0.85)),
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
                                .size(theme::text::META)
                                .color(th.error),
                        );
                    } else if card.error.is_some() {
                        ui.label(
                            RichText::new("Error al abrir")
                                .size(theme::text::META)
                                .color(th.error),
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
                                    .size(theme::text::META)
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
    // La tarjeta se pinta a mano, pero el foco y Enter/Espacio los aporta un
    // botón transparente que cubre el área. Así la grilla es navegable con
    // teclado sin reescribir el layout de portada/título/autor.
    let id = ui.make_persistent_id(("library-card", card.index));
    let mut response = ui.interact(rect, id, Sense::click());
    if focus {
        response.request_focus();
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            theme::radius::LG,
            Stroke::new(2.0, th.accent),
            egui::StrokeKind::Inside,
        );
    } else if response.hovered() {
        ui.painter().rect_stroke(
            rect.expand(3.0),
            theme::radius::LG + 1.0,
            Stroke::new(1.0, th.accent.gamma_multiply(0.45)),
            egui::StrokeKind::Inside,
        );
    }
    let activated = response.clicked()
        || (response.has_focus()
            && ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)));
    if activated {
        action = Some(if card.missing || card.error.is_some() {
            CardAction::Recover
        } else {
            CardAction::Open
        });
    }
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    if let Some(error) = &card.error {
        response = response.on_hover_text(format!(
            "{error}\n\nClic para ubicar el archivo o quitarlo de la biblioteca."
        ));
    } else if card.missing {
        response = response.on_hover_text(
            "El archivo ya no está.\n\nClic para ubicarlo o quitarlo de la biblioteca.",
        );
    } else if card.duplicate_count > 0 {
        response = response.on_hover_text(format!(
            "Posible duplicado: {} libros con el mismo título y autor.\n\nClic para abrir; menú derecho para mostrar en Finder o quitarlo de la biblioteca.",
            card.duplicate_count
        ));
    }
    response.context_menu(|ui| {
        if !card.missing && card.error.is_none() && ui.button("Abrir").clicked() {
            action = Some(CardAction::Open);
            ui.close();
        }
        if ui.button("Mostrar en Finder").clicked() {
            reveal_in_finder(&card.path);
            ui.close();
        }
        if (card.missing || card.error.is_some()) && ui.button("Ubicar archivo…").clicked() {
            action = Some(CardAction::Locate);
            ui.close();
        }
        if !card.missing
            && card.error.is_none()
            && ui.button("Editar metadatos y portada…").clicked()
        {
            action = Some(CardAction::Edit);
            ui.close();
        }
        ui.separator();
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
    blocks: &[Block],
    chapter: usize,
    link_click: &mut Option<LinkClick>,
    external_link: &mut Option<String>,
    selection: &mut Option<Selection>,
    extend_to: &mut Option<Anchor>,
    geoms: &mut Vec<BlockGeom>,
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
                let resp = add_selectable_text(
                    ui,
                    j2,
                    &args,
                    blocks,
                    link_click,
                    external_link,
                    selection,
                    extend_to,
                    geoms,
                    reader_focus,
                );
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
            add_selectable_text(
                ui,
                job,
                &args,
                blocks,
                link_click,
                external_link,
                selection,
                extend_to,
                geoms,
                reader_focus,
            );
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

    fn seg(block: usize, start: usize, end: usize) -> Segment {
        Segment { block, start, end }
    }

    #[test]
    fn destacado_respeta_offsets_y_reancla_por_cita() {
        let blocks = vec![paragraph("Hola mundo"), paragraph("Otra frase distinta")];
        let exacto = annotation(0, 5, 10, "mundo");
        assert_eq!(
            resolve_annotation(&exacto, &blocks),
            Some(vec![seg(0, 5, 10)])
        );

        // El bloque guardado cambió, pero la cita sigue en el capítulo.
        let movido = annotation(1, 0, 5, "mundo");
        assert_eq!(
            resolve_annotation(&movido, &blocks),
            Some(vec![seg(0, 5, 10)])
        );

        // Offsets viejos dentro del bloque correcto: se reancla por cita.
        let corrido = annotation(0, 0, 9, "frase distinta");
        assert_eq!(
            resolve_annotation(&corrido, &blocks),
            Some(vec![seg(1, 5, 19)])
        );
    }

    #[test]
    fn destacado_multi_bloque_respeta_offsets() {
        let blocks = vec![
            paragraph("Hola mundo"),
            paragraph("Segunda frase"),
            paragraph("Tercera parte"),
        ];
        let mut a = annotation(0, 5, 7, "");
        a.end_block = Some(2);
        a.parts = vec!["mundo".into(), "Segunda frase".into(), "Tercera".into()];
        a.quote = "mundo\nSegunda frase\nTercera".into();
        assert_eq!(
            resolve_annotation(&a, &blocks),
            Some(vec![seg(0, 5, 10), seg(1, 0, 13), seg(2, 0, 7)])
        );
    }

    #[test]
    fn destacado_multi_bloque_se_reancla_por_partes() {
        let blocks = vec![
            paragraph("Prólogo: Hola mundo"),
            paragraph("Otra vez Segunda frase"),
            paragraph("Tercera parte final"),
        ];
        let mut a = annotation(0, 5, 5, "");
        a.end_block = Some(2);
        a.parts = vec!["mundo".into(), "Segunda frase".into(), "Tercera".into()];
        a.quote = "mundo\nSegunda frase\nTercera".into();
        assert_eq!(
            resolve_annotation(&a, &blocks),
            Some(vec![seg(0, 14, 19), seg(1, 9, 22), seg(2, 0, 7)])
        );
    }

    #[test]
    fn destacado_multi_bloque_con_coincidencia_parcial_no_se_dibuja() {
        let blocks = vec![paragraph("Hola mundo"), paragraph("Texto distinto")];
        let mut a = annotation(0, 5, 5, "");
        a.end_block = Some(1);
        a.parts = vec!["mundo".into(), "Segunda frase".into()];
        a.quote = "mundo\nSegunda frase".into();
        assert_eq!(resolve_annotation(&a, &blocks), None);
    }

    #[test]
    fn seleccion_multi_bloque_arma_partes_y_omite_bloques_vacios() {
        let blocks = vec![
            paragraph("Hola mundo"),
            paragraph(""),
            paragraph("Segunda frase"),
        ];
        let selection = selection_from_anchors(
            2,
            &blocks,
            Anchor {
                block: 0,
                offset: 5,
            },
            Anchor {
                block: 2,
                offset: 7,
            },
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(10.0, 10.0)),
            egui::Id::new("test"),
        )
        .expect("selección válida");
        assert_eq!(selection.quote, "mundo\nSegunda");
        assert_eq!(selection.parts, vec!["mundo", "Segunda"]);
        assert!(selection.is_multi_block());
        assert_eq!(selection.start_block(), 0);
        assert_eq!(selection.end_block(), 2);
        assert_eq!(
            selection_segments(&selection, &blocks),
            vec![seg(0, 5, 10), seg(2, 0, 7)]
        );
    }

    #[test]
    fn seleccion_multi_bloque_se_convierte_en_anotacion() {
        let blocks = vec![paragraph("Hola mundo"), paragraph("Segunda frase")];
        let selection = selection_from_anchors(
            0,
            &blocks,
            Anchor {
                block: 0,
                offset: 5,
            },
            Anchor {
                block: 1,
                offset: 7,
            },
            egui::Rect::NOTHING,
            egui::Id::new("test"),
        )
        .expect("selección válida");
        let a = annotation_from_selection(&selection, "/x.epub".into(), "nota".into(), 2);
        assert_eq!(a.book, "/x.epub");
        assert_eq!(a.chapter, 0);
        assert_eq!(a.block, 0);
        assert_eq!(a.start, 5);
        assert_eq!(a.end, 7);
        assert_eq!(a.end_block(), 1);
        assert!(a.is_multi_block());
        assert_eq!(a.parts, vec!["mundo", "Segunda"]);
        assert_eq!(a.quote, "mundo\nSegunda");
        assert_eq!(a.note, "nota");
        assert_eq!(a.color, 2);
    }

    #[test]
    fn seleccion_solo_espacios_no_es_valida() {
        let blocks = vec![paragraph("   "), paragraph("")];
        assert!(selection_from_anchors(
            0,
            &blocks,
            Anchor {
                block: 0,
                offset: 0,
            },
            Anchor {
                block: 1,
                offset: 0,
            },
            egui::Rect::NOTHING,
            egui::Id::new("test"),
        )
        .is_none());
    }

    #[test]
    fn quote_inline_colapsa_espacios_y_saltos() {
        assert_eq!(quote_inline("uno\n  dos\t tres"), "uno dos tres");
    }

    #[test]
    fn filtro_de_notas_busca_cita_nota_libro_y_capitulo() {
        assert!(note_matches("", "x", "", "", ""));
        assert!(note_matches("mundo", "Hola mundo", "", "", ""));
        assert!(note_matches("ironia", "Hola", "La ironía", "", ""));
        assert!(note_matches("austen", "Hola", "", "Jane Austen", ""));
        assert!(note_matches("capitulo 1", "Hola", "", "", "Capítulo 1"));
        assert!(!note_matches("zzz", "Hola", "nota", "libro", "cap"));
    }

    #[test]
    fn preview_text_usa_el_ancla_y_salta_bloques_vacios() {
        let mut blocks = vec![paragraph(""), paragraph("Nota uno"), paragraph("Otra nota")];
        blocks[0].anchor = Some("fn1".into());
        assert_eq!(preview_text(&blocks, Some("fn1"), 600), "Nota uno");
        assert_eq!(preview_text(&blocks, Some("no-existe"), 600), "Nota uno");
        assert_eq!(preview_text(&blocks, None, 600), "Nota uno");
        assert_eq!(preview_text(&[], None, 600), "");
    }

    #[test]
    fn preview_text_salta_encabezado_y_trunca() {
        let heading = Block {
            kind: Kind::Heading(1),
            runs: vec![crate::xhtml::Run {
                text: "Notas".into(),
                ..Default::default()
            }],
            src: None,
            alt: String::new(),
            anchor: Some("notas".into()),
        };
        let blocks = vec![heading, paragraph("Una nota breve")];
        assert_eq!(preview_text(&blocks, Some("notas"), 600), "Una nota breve");
        assert_eq!(preview_text(&blocks, Some("notas"), 4), "Una…");
    }

    #[test]
    fn detecta_marcadores_de_nota() {
        for label in ["1", "12", "iv", "IV", "[2]", "*"] {
            assert!(looks_like_note_marker(label), "{label}");
        }
        for label in ["", "nota", "ver", "1234567"] {
            assert!(!looks_like_note_marker(label), "{label}");
        }
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
    fn historial_resuelve_bloque_y_rango_por_cita() {
        let blocks = vec![paragraph("Hola mundo"), paragraph("the sleep of reason")];
        assert_eq!(
            resolve_history_range(&blocks, 0, "the sleep of reason"),
            Some((1, 0, 19))
        );
        assert_eq!(resolve_history_range(&blocks, 0, "mundo"), Some((0, 5, 10)));
        assert_eq!(resolve_history_range(&blocks, 0, "zzz"), None);
    }

    #[test]
    fn historial_multi_bloque_cae_al_prefijo_de_palabras() {
        let blocks = vec![paragraph("Hola mundo"), paragraph("Otra frase distinta")];
        assert_eq!(
            resolve_history_range(&blocks, 0, "Hola mundo Otra frase distinta"),
            Some((0, 0, 10))
        );
        assert_eq!(resolve_history_range(&blocks, 0, "zzz yyy"), None);
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

    #[test]
    fn contexto_recorta_alrededor_de_la_seleccion() {
        let text = "a".repeat(1_000);
        let context = context_around(&text, 400, 450, 100);
        assert!(context.starts_with('…'));
        assert!(context.ends_with('…'));
        assert!(context.chars().count() <= 102);
        assert_eq!(context_around("hola", 0, 4, 100), "hola");
    }

    #[test]
    fn filtro_por_serie_y_sin_serie() {
        let mut book = BookEntry {
            series: Some("Saga".into()),
            ..BookEntry::default()
        };
        let duplicates = HashMap::new();
        assert!(matches_filter(
            &LibraryFilter::Series("Saga".into()),
            &book,
            &duplicates
        ));
        assert!(matches_filter(
            &LibraryFilter::Series("saga".into()),
            &book,
            &duplicates
        ));
        assert!(!matches_filter(
            &LibraryFilter::Series("Otra".into()),
            &book,
            &duplicates
        ));
        assert!(!matches_filter(
            &LibraryFilter::NoSeries,
            &book,
            &duplicates
        ));

        book.series = None;
        assert!(matches_filter(&LibraryFilter::NoSeries, &book, &duplicates));
        assert!(!matches_filter(
            &LibraryFilter::Series("Saga".into()),
            &book,
            &duplicates
        ));
    }

    #[test]
    fn grilla_arma_filas_y_grupos_por_serie() {
        let no_series = vec![None, None, None];
        assert_eq!(
            build_grid_items(&no_series, 2, false),
            vec![
                GridItem::Row { start: 0, end: 2 },
                GridItem::Row { start: 2, end: 3 },
            ]
        );

        let series = vec![
            Some("Saga".into()),
            Some("saga".into()),
            Some("Otra".into()),
            None,
            None,
        ];
        assert_eq!(
            build_grid_items(&series, 2, true),
            vec![
                GridItem::Header {
                    label: "Saga".into(),
                    count: 2
                },
                GridItem::Row { start: 0, end: 2 },
                GridItem::Header {
                    label: "Otra".into(),
                    count: 1
                },
                GridItem::Row { start: 2, end: 3 },
                GridItem::Header {
                    label: "Sin serie".into(),
                    count: 2
                },
                GridItem::Row { start: 3, end: 5 },
            ]
        );
    }

    #[test]
    fn marcador_compara_ancla_y_fraccion() {
        let a = Progress {
            chapter: 4,
            ratio: 0.2,
            anchor: Some(10),
            anchor_ratio: Some(0.25),
            legacy_offset: None,
        };
        let close = Progress {
            anchor_ratio: Some(0.255),
            ..a
        };
        let far = Progress {
            anchor_ratio: Some(0.9),
            ..a
        };
        let other_anchor = Progress {
            anchor: Some(11),
            ..a
        };
        let other_chapter = Progress { chapter: 5, ..a };
        assert!(Lector::bookmark_matches(&a, &close));
        assert!(!Lector::bookmark_matches(&a, &far));
        assert!(!Lector::bookmark_matches(&a, &other_anchor));
        assert!(!Lector::bookmark_matches(&a, &other_chapter));

        let no_anchor = Progress {
            chapter: 1,
            ratio: 0.4,
            anchor: None,
            anchor_ratio: None,
            legacy_offset: None,
        };
        let close_ratio = Progress {
            ratio: 0.405,
            ..no_anchor
        };
        assert!(Lector::bookmark_matches(&no_anchor, &close_ratio));
    }

    #[test]
    fn detecta_duplicados_por_titulo_y_autor() {
        let first = BookEntry {
            title: "Cien años de soledad".into(),
            author: "Gabriel García Márquez".into(),
            ..BookEntry::default()
        };
        let second = BookEntry {
            title: "cien anos de soledad".into(),
            author: "gabriel garcia marquez".into(),
            ..BookEntry::default()
        };
        let unique = BookEntry {
            title: "Otro libro".into(),
            author: "Otra autora".into(),
            ..BookEntry::default()
        };
        let empty = BookEntry::default();
        let books = vec![first.clone(), second.clone(), unique.clone(), empty];

        let counts = duplicate_counts(&books);
        assert_eq!(counts.values().filter(|count| **count > 1).count(), 1);
        assert_eq!(
            counts.get(&duplicate_key(&first).unwrap()).copied(),
            Some(2)
        );
        assert!(matches_filter(&LibraryFilter::Duplicates, &first, &counts));
        assert!(matches_filter(&LibraryFilter::Duplicates, &second, &counts));
        assert!(!matches_filter(
            &LibraryFilter::Duplicates,
            &unique,
            &counts
        ));
    }

    #[test]
    fn etiqueta_de_serie_recorta_nombres_largos() {
        let label = LibraryFilter::Series("Una serie con un nombre larguísimo".into()).label();
        assert!(label.starts_with("Serie · "));
        assert!(label.ends_with('…'));
    }
}
