//! Test de integración de punta a punta sobre el fixture versionado.
//!
//! Es el contrato mínimo que cualquier cambio en `epub.rs` / `xhtml.rs` debe
//! respetar. No necesita GUI ni libros externos.

use std::path::PathBuf;

use candil::epub::Book;
use candil::xhtml::Kind;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mini.epub")
}

fn abrir() -> Book {
    Book::open(fixture()).expect("el fixture debe abrir")
}

#[test]
fn expone_metadatos_y_estructura() {
    let mut book = abrir();
    assert_eq!(book.title, "Libro de prueba");
    assert_eq!(book.author, "Autora Ficticia");
    assert_eq!(book.lang, "es");
    assert_eq!(book.chapter_count(), 3);
    assert_eq!(book.toc.len(), 3);
    assert_eq!(book.toc[0].spine, 0);
    assert_eq!(book.toc[1].spine, 1);
    // El índice apunta a cap3.xhtml#final: el ancla no debe romper el mapeo.
    assert_eq!(book.toc[2].spine, 2);
    assert_eq!(book.toc[2].anchor.as_deref(), Some("final"));
    assert_eq!(book.chapter_title(0), "Uno");
}

#[test]
fn ningun_capitulo_queda_vacio() {
    let mut book = abrir();
    for i in 0..book.chapter_count() {
        let blocks = book.chapter_blocks(i);
        let words: usize = blocks
            .iter()
            .map(|b| b.text().split_whitespace().count())
            .sum();
        assert!(words > 0, "el capítulo {i} salió vacío");
    }
}

#[test]
fn capitulo_html_invalido_pasa_por_el_parser_tolerante() {
    let mut book = abrir();
    let blocks = book.chapter_blocks(1);
    let texto = blocks
        .iter()
        .map(|b| b.text())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(texto.contains("no es XML válido"), "{texto}");
    assert!(texto.contains("negrita"), "{texto}");
}

#[test]
fn preserva_estilos_inline_y_bloques() {
    let mut book = abrir();
    let blocks = book.chapter_blocks(0);
    assert!(blocks.iter().any(|b| matches!(b.kind, Kind::Heading(1))));
    assert!(blocks.iter().any(|b| b.kind == Kind::Quote));
    assert!(blocks.iter().any(|b| matches!(b.kind, Kind::Numbered(2))));
    assert!(blocks
        .iter()
        .flat_map(|b| b.runs.iter())
        .any(|r| r.italic && r.text.contains("énfasis")));
    assert!(blocks
        .iter()
        .flat_map(|b| b.runs.iter())
        .any(|r| r.bold && r.text.contains("negrita")));
}

#[test]
fn las_imagenes_se_resuelven_dentro_del_zip() {
    let mut book = abrir();
    let blocks = book.chapter_blocks(0);
    let img = blocks
        .iter()
        .find(|b| b.kind == Kind::Image)
        .expect("el fixture tiene una imagen");
    let src = img.src.clone().expect("src resuelto");
    assert_eq!(src, "OEBPS/Images/punto.png");
    assert!(
        book.resource(&src).is_some(),
        "el recurso debe estar en el zip"
    );
}

#[test]
fn archivo_inexistente_da_error_legible() {
    let Err(e) = Book::open("/ningun/lado/existe.epub") else {
        panic!("debería fallar");
    };
    assert!(e.contains("no pude leer"), "{e}");
}

#[test]
fn portada_declarada_y_fallback_de_imagen() {
    let mut mini = abrir();
    assert!(mini.cover_bytes().is_some(), "mini declara cover-image");

    let mut edge =
        Book::open(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/edge.epub"))
            .expect("edge debe abrir");
    assert!(
        edge.cover_bytes().is_some(),
        "edge no declara portada: debe caer al <svg><image>"
    );
}

#[test]
fn notas_resuelven_anclas_y_texto() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/notes.epub");
    let mut book = Book::open(path).expect("notes.epub debe abrir");
    assert_eq!(book.chapter_count(), 2);
    assert_eq!(
        book.link_target("#fn1", 0),
        Some((0, Some("fn1".to_string())))
    );
    assert_eq!(
        book.link_target("notas.xhtml#fn2", 0),
        Some((1, Some("fn2".to_string())))
    );
    assert!(book.link_target("https://example.com", 0).is_none());

    let blocks = book.chapter_blocks(0);
    let links: Vec<(String, String)> = blocks
        .iter()
        .flat_map(|block| {
            block.runs.iter().filter_map(|run| {
                run.href
                    .as_ref()
                    .map(|href| (run.text.trim().to_string(), href.clone()))
            })
        })
        .collect();
    assert_eq!(
        links,
        vec![
            ("1".to_string(), "#fn1".to_string()),
            ("2".to_string(), "notas.xhtml#fn2".to_string())
        ]
    );
    let note = blocks
        .iter()
        .find(|block| block.anchor.as_deref() == Some("fn1"))
        .expect("nota en el mismo capítulo");
    assert!(note.text().contains("Nota al pie en el mismo capítulo"));

    let notes = book.chapter_blocks(1);
    let note = notes
        .iter()
        .find(|block| block.anchor.as_deref() == Some("fn2"))
        .expect("nota en otro documento");
    assert!(note.text().contains("Nota dos"));
}

#[test]
fn busqueda_encuentra_con_acentos_y_mapea_al_bloque_real() {
    let mut book = abrir();
    let index = candil::search::build(&mut book);
    let (hits, truncated) = candil::search::search(&index, "enfasis", 10);
    assert!(!truncated);
    assert!(!hits.is_empty(), "el fixture contiene «énfasis»");

    let hit = &hits[0];
    assert_eq!(hit.chapter, 0);
    let blocks = book.chapter_blocks(hit.chapter);
    let text = blocks[hit.block].text();
    let slice: String = text
        .chars()
        .skip(hit.start)
        .take(hit.end - hit.start)
        .collect();
    assert_eq!(slice.to_lowercase(), "énfasis");
    assert_eq!(hit.matched.to_lowercase(), "énfasis");
}
