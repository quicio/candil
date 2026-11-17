//! API C para Flutter. Las respuestas viajan como JSON UTF-8; la UI no depende
//! de layouts de Rust ni de `flutter_rust_bridge`.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::epub::Book;
use crate::search::{QueryMode, SearchIndex};
use crate::xhtml::{Block, Kind};

pub struct BookHandle {
    book: Mutex<Book>,
    index: Mutex<Option<Arc<SearchIndex>>>,
}

#[repr(C)]
pub struct CandilBook {
    pub handle: *mut BookHandle,
    pub error: *mut c_char,
}

#[repr(C)]
pub struct CandilBuffer {
    pub data: *mut u8,
    pub len: usize,
    pub cap: usize,
}

#[derive(Serialize)]
struct TocDto {
    title: String,
    spine: usize,
    anchor: Option<String>,
}

#[derive(Serialize)]
struct MetadataDto {
    path: String,
    title: String,
    author: String,
    lang: String,
    series: Option<String>,
    series_index: Option<f32>,
    chapters: usize,
    toc: Vec<TocDto>,
}

#[derive(Serialize)]
struct RunDto {
    text: String,
    bold: bool,
    italic: bool,
    small: bool,
    link: bool,
    href: Option<String>,
}

#[derive(Serialize)]
struct BlockDto {
    kind: &'static str,
    level: Option<u8>,
    number: Option<u32>,
    text: String,
    src: Option<String>,
    alt: String,
    anchor: Option<String>,
    runs: Vec<RunDto>,
}

#[derive(Serialize)]
struct ChapterDto {
    index: usize,
    title: String,
    blocks: Vec<BlockDto>,
}

#[derive(Serialize)]
struct HitDto {
    chapter: usize,
    block: usize,
    start: usize,
    end: usize,
    before: String,
    matched: String,
    after: String,
}

struct HitBuf {
    hits: Vec<HitDto>,
    truncated: bool,
}

fn string_ptr(value: String) -> *mut c_char {
    CString::new(value)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn error_json(message: impl AsRef<str>) -> *mut c_char {
    let payload = serde_json::json!({ "ok": false, "error": message.as_ref() });
    string_ptr(payload.to_string())
}

fn ok_json<T: Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(text) => string_ptr(text),
        Err(error) => error_json(format!("No pude serializar la respuesta: {error}")),
    }
}

fn buffer_from_vec(bytes: Vec<u8>) -> *mut CandilBuffer {
    let mut bytes = bytes;
    let buffer = CandilBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        cap: bytes.capacity(),
    };
    std::mem::forget(bytes);
    Box::into_raw(Box::new(buffer))
}

fn empty_buffer() -> *mut CandilBuffer {
    buffer_from_vec(Vec::new())
}

fn block_dto(block: Block) -> BlockDto {
    let (kind, level, number) = match block.kind {
        Kind::Para => ("para", None, None),
        Kind::Heading(level) => ("heading", Some(level), None),
        Kind::Quote => ("quote", None, None),
        Kind::Bullet => ("bullet", None, None),
        Kind::Numbered(number) => ("numbered", None, Some(number)),
        Kind::Rule => ("rule", None, None),
        Kind::Pre => ("pre", None, None),
        Kind::Caption => ("caption", None, None),
        Kind::Image => ("image", None, None),
    };
    let text = block.text();
    let runs = block
        .runs
        .into_iter()
        .map(|run| RunDto {
            text: run.text,
            bold: run.bold,
            italic: run.italic,
            small: run.small,
            link: run.link,
            href: run.href,
        })
        .collect();
    BlockDto {
        kind,
        level,
        number,
        text,
        src: block.src,
        alt: block.alt,
        anchor: block.anchor,
        runs,
    }
}

#[no_mangle]
pub extern "C" fn candil_open(path: *const c_char) -> CandilBook {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if path.is_null() {
            return Err("Ruta vacía.".to_string());
        }
        let path = unsafe { CStr::from_ptr(path) }
            .to_str()
            .map_err(|_| "La ruta no es UTF-8.".to_string())?;
        let book = Book::open(path)?;
        Ok(BookHandle {
            book: Mutex::new(book),
            index: Mutex::new(None),
        })
    }));
    match result {
        Ok(Ok(handle)) => CandilBook {
            handle: Box::into_raw(Box::new(handle)),
            error: std::ptr::null_mut(),
        },
        Ok(Err(error)) => CandilBook {
            handle: std::ptr::null_mut(),
            error: string_ptr(error),
        },
        Err(_) => CandilBook {
            handle: std::ptr::null_mut(),
            error: string_ptr("El parser de EPUB falló inesperadamente.".into()),
        },
    }
}

#[no_mangle]
pub extern "C" fn candil_metadata(handle: *mut BookHandle) -> *mut c_char {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        return error_json("Libro cerrado.");
    };
    let Ok(book) = handle.book.lock() else {
        return error_json("No pude acceder al libro.");
    };
    let metadata = MetadataDto {
        path: book.path.to_string_lossy().to_string(),
        title: book.title.clone(),
        author: book.author.clone(),
        lang: book.lang.clone(),
        series: book.series.clone(),
        series_index: book.series_index,
        chapters: book.chapter_count(),
        toc: book
            .toc
            .iter()
            .map(|entry| TocDto {
                title: entry.title.clone(),
                spine: entry.spine,
                anchor: entry.anchor.clone(),
            })
            .collect(),
    };
    ok_json(&metadata)
}

#[no_mangle]
pub extern "C" fn candil_chapter(handle: *mut BookHandle, index: usize) -> *mut c_char {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        return error_json("Libro cerrado.");
    };
    let Ok(mut book) = handle.book.lock() else {
        return error_json("No pude acceder al libro.");
    };
    let title = book.chapter_title(index);
    let blocks = book
        .chapter_blocks(index)
        .into_iter()
        .map(block_dto)
        .collect();
    ok_json(&ChapterDto {
        index,
        title,
        blocks,
    })
}

#[no_mangle]
pub extern "C" fn candil_cover(handle: *mut BookHandle) -> *mut CandilBuffer {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        return empty_buffer();
    };
    let Ok(mut book) = handle.book.lock() else {
        return empty_buffer();
    };
    match book.cover_bytes() {
        Some(bytes) => buffer_from_vec(bytes),
        None => empty_buffer(),
    }
}

#[no_mangle]
pub extern "C" fn candil_resource(
    handle: *mut BookHandle,
    path: *const c_char,
) -> *mut CandilBuffer {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        return empty_buffer();
    };
    if path.is_null() {
        return empty_buffer();
    }
    let Ok(path) = (unsafe { CStr::from_ptr(path) }).to_str() else {
        return empty_buffer();
    };
    let Ok(mut book) = handle.book.lock() else {
        return empty_buffer();
    };
    match book.resource(path) {
        Some(bytes) => buffer_from_vec(bytes),
        None => empty_buffer(),
    }
}

#[no_mangle]
pub extern "C" fn candil_search(
    handle: *mut BookHandle,
    query: *const c_char,
    mode: u8,
    max: usize,
) -> *mut c_char {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        return error_json("Libro cerrado.");
    };
    if query.is_null() {
        return error_json("Consulta vacía.");
    }
    let Ok(query) = (unsafe { CStr::from_ptr(query) }).to_str() else {
        return error_json("La consulta no es UTF-8.");
    };
    let mode = match mode {
        1 => QueryMode::AllWords,
        2 => QueryMode::AnyWord,
        _ => QueryMode::Phrase,
    };
    let index = {
        let Ok(mut slot) = handle.index.lock() else {
            return error_json("No pude acceder al índice.");
        };
        if let Some(index) = slot.as_ref() {
            Arc::clone(index)
        } else {
            let Ok(mut book) = handle.book.lock() else {
                return error_json("No pude acceder al libro.");
            };
            let index = Arc::new(crate::search::build(&mut book));
            *slot = Some(Arc::clone(&index));
            index
        }
    };
    let (hits, truncated) = crate::search::search_mode(&index, query, mode, max);
    let result = HitBuf {
        hits: hits
            .into_iter()
            .map(|hit| HitDto {
                chapter: hit.chapter,
                block: hit.block,
                start: hit.start,
                end: hit.end,
                before: hit.before,
                matched: hit.matched,
                after: hit.after,
            })
            .collect(),
        truncated,
    };
    let payload = serde_json::json!({
        "ok": true,
        "hits": result.hits,
        "truncated": result.truncated,
    });
    string_ptr(payload.to_string())
}

#[no_mangle]
pub extern "C" fn candil_close(handle: *mut BookHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub extern "C" fn candil_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

#[no_mangle]
pub extern "C" fn candil_free_buffer(ptr: *mut CandilBuffer) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let buffer = Box::from_raw(ptr);
        if !buffer.data.is_null() {
            drop(Vec::from_raw_parts(buffer.data, buffer.len, buffer.cap));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    fn fixture(name: &str) -> CString {
        CString::new(format!(
            "{}/tests/fixtures/{name}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap()
    }

    #[test]
    fn abre_metadatos_capitulo_y_cierra() {
        let path = fixture("mini.epub");
        let opened = candil_open(path.as_ptr());
        assert!(!opened.handle.is_null());
        assert!(opened.error.is_null());

        let metadata = candil_metadata(opened.handle);
        let metadata_text = unsafe { CStr::from_ptr(metadata) }
            .to_str()
            .unwrap()
            .to_string();
        assert!(metadata_text.contains("\"chapters\""));
        assert!(metadata_text.contains("Libro de prueba"));
        candil_free_string(metadata);

        let chapter = candil_chapter(opened.handle, 0);
        let chapter_text = unsafe { CStr::from_ptr(chapter) }
            .to_str()
            .unwrap()
            .to_string();
        assert!(chapter_text.contains("\"blocks\""));
        candil_free_string(chapter);

        let cover = candil_cover(opened.handle);
        assert!(!cover.is_null());
        candil_free_buffer(cover);

        candil_close(opened.handle);
    }

    #[test]
    fn ruta_invalida_devuelve_error() {
        let path = CString::new("/no/existe.epub").unwrap();
        let opened = candil_open(path.as_ptr());
        assert!(opened.handle.is_null());
        assert!(!opened.error.is_null());
        let error = unsafe { CStr::from_ptr(opened.error) }
            .to_str()
            .unwrap()
            .to_string();
        assert!(error.contains("No pude") || error.contains("no pude"));
        candil_free_string(opened.error);
    }
}
