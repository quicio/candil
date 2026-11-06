//! Bordes del parser que la auditoría de M1 encontró en libros reales.
//!
//! Se apoyan en `tests/fixtures/edge.epub` (regenerable con
//! `python3 scripts/make_fixture.py`) para probar el camino completo:
//! zip → XHTML → `Vec<Block>` → recursos resueltos.

use std::path::PathBuf;

use lector::epub::Book;
use lector::xhtml::Kind;

fn edge() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/edge.epub")
}

#[test]
fn espacios_entidades_pre_tablas_y_svg() {
    let mut book = Book::open(edge()).expect("el fixture de bordes debe abrir");
    assert_eq!(book.title, "Bordes");
    assert_eq!(book.chapter_count(), 1);
    assert_eq!(book.toc.len(), 1);

    let blocks = book.chapter_blocks(0);
    let texto = blocks
        .iter()
        .map(|b| b.text())
        .collect::<Vec<_>>()
        .join(" | ");

    // H1: un espacio entre dos runs inline no desaparece.
    assert!(texto.contains("Hola mundo."), "{texto}");
    // H3: entidades Latin-1 y griegas.
    assert!(
        texto.contains("caf\u{e9} y ni\u{f1}o, \u{2264} \u{3c9}."),
        "{texto}"
    );

    // H4: <pre> conserva el salto y la indentación interna.
    let pre = blocks
        .iter()
        .find(|b| b.kind == Kind::Pre)
        .expect("el fixture tiene un <pre>");
    assert_eq!(pre.text(), "def f(x):\n    return x");

    // H4: las celdas y definiciones no se pegan.
    assert!(blocks.iter().any(|b| b.text() == "celda uno"), "{texto}");
    assert!(blocks.iter().any(|b| b.text() == "celda dos"), "{texto}");
    assert!(blocks.iter().any(|b| b.text() == "t\u{e9}rmino"), "{texto}");
    assert!(
        blocks.iter().any(|b| b.text() == "definici\u{f3}n"),
        "{texto}"
    );
    assert!(!texto.contains("celda unocelda"), "{texto}");

    // H5: el <svg><image> se convierte en una imagen resoluble del zip.
    let imagen = blocks
        .iter()
        .find(|b| b.kind == Kind::Image)
        .expect("el fixture tiene un SVG con imagen");
    let src = imagen.src.as_deref().expect("src resuelto");
    assert_eq!(src, "OEBPS/Images/punto.png");
    assert!(
        book.resource(src).is_some(),
        "la imagen del SVG debe estar en el zip"
    );
}
