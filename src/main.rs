use std::path::PathBuf;

use lector::app;
use lector::epub;
use lector::xhtml;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if let Some(pos) = args.iter().position(|a| a == "--inspect") {
        let path = args.get(pos + 1).cloned().unwrap_or_default();
        inspect(&path);
        return Ok(());
    }

    let mut path: Option<PathBuf> = None;
    let mut dev = app::DevOpts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--shot" => {
                dev.shot = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            "--chapter" => {
                dev.chapter = args.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "--panel" => {
                dev.panel = args.get(i + 1).cloned();
                i += 2;
            }
            other => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
        }
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([980.0, 780.0])
            .with_min_inner_size([420.0, 360.0])
            .with_app_id("lector"),
        ..Default::default()
    };

    eframe::run_native(
        "lector",
        options,
        Box::new(move |cc| Ok(Box::new(app::Lector::new(cc, path, dev)))),
    )
}

/// Modo diagnóstico: valida el parseo sin abrir ventana.
fn inspect(path: &str) {
    if path.is_empty() {
        eprintln!("uso: lector --inspect <libro.epub>");
        return;
    }
    match epub::Book::open(path) {
        Ok(mut book) => {
            println!("archivo : {}", book.path.display());
            println!("título  : {}", book.title);
            println!("autor   : {}", book.author);
            println!("idioma  : {}", book.lang);
            println!("capítulos: {}", book.spine.len());
            println!("índice  : {} entradas", book.toc.len());
            for e in book.toc.iter().take(12) {
                println!("   [{:>3}] {}", e.spine, e.title);
            }
            println!("\n-- capítulos --");
            let mut empty = Vec::new();
            let mut images = 0usize;
            let mut images_ok = 0usize;
            for i in 0..book.spine.len() {
                let blocks = book.chapter_blocks(i);
                let words: usize = blocks
                    .iter()
                    .map(|b| b.text().split_whitespace().count())
                    .sum();
                for b in blocks.iter().filter(|b| b.kind == xhtml::Kind::Image) {
                    images += 1;
                    if let Some(src) = &b.src {
                        if book.resource(src).is_some() {
                            images_ok += 1;
                        }
                    }
                }
                let first = blocks
                    .iter()
                    .find(|b| !b.text().trim().is_empty())
                    .map(|b| {
                        let t = b.text();
                        t.chars().take(60).collect::<String>()
                    })
                    .unwrap_or_default();
                println!(
                    "[{:>3}] {:>5} bloques {:>6} palabras  {}",
                    i,
                    blocks.len(),
                    words,
                    first
                );
                if blocks.is_empty() {
                    empty.push(i);
                }
            }
            if !empty.is_empty() {
                println!("\ncapítulos vacíos: {:?}", empty);
            }
            println!("\nimágenes: {images_ok}/{images} resolubles");
        }
        Err(e) => eprintln!("error: {e}"),
    }
}
