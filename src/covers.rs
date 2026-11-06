//! Portadas: miniatura (sin agrandar), ruta estable y escritura en `covers/`.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::GenericImageView;

/// Alto de la miniatura. Nunca se agranda una portada más chica.
pub const COVER_HEIGHT: u32 = 320;

/// Decodifica la imagen, la reduce a `height` de alto (ancho proporcional) y
/// devuelve PNG. Si la imagen original es más chica, la deja como está.
pub fn thumbnail(bytes: &[u8], height: u32) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        return None;
    }
    let target_h = height.min(h);
    let target_w = ((w as f64 * target_h as f64 / h as f64).round() as u32).max(1);
    let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3);
    let mut out = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .ok()?;
    Some(out)
}

/// Ruta estable de la portada cacheada: FNV-1a sobre la ruta canónica del libro.
/// No usa `DefaultHasher` porque su algoritmo no está garantizado entre versiones.
pub fn cover_path_for(dir: &Path, book_path: &Path) -> PathBuf {
    dir.join(format!("{:016x}.png", stable_hash(book_path)))
}

fn stable_hash(path: &Path) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub fn write_cover(path: &Path, png: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, png)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "lector-covers-{tag}-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn png_of(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(w, h, image::Rgba([20, 40, 60, 255]));
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn miniatura_no_agranda() {
        let out = thumbnail(&png_of(10, 20), COVER_HEIGHT).expect("miniatura");
        let img = image::load_from_memory(&out).expect("png");
        assert_eq!(img.dimensions(), (10, 20));
    }

    #[test]
    fn miniatura_reduce_a_320() {
        let out = thumbnail(&png_of(400, 600), COVER_HEIGHT).expect("miniatura");
        let img = image::load_from_memory(&out).expect("png");
        let (w, h) = img.dimensions();
        assert_eq!(h, COVER_HEIGHT);
        assert!(w < 400 && w > 0, "{w}");
    }

    #[test]
    fn ruta_de_portada_es_estable() {
        let a = cover_path_for(Path::new("/tmp/covers"), Path::new("/libros/a.epub"));
        let b = cover_path_for(Path::new("/tmp/covers"), Path::new("/libros/a.epub"));
        let c = cover_path_for(Path::new("/tmp/covers"), Path::new("/libros/b.epub"));
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.file_name().unwrap().to_string_lossy().ends_with(".png"));
    }

    #[test]
    fn escribe_la_portada_en_disco() {
        let dir = temp_dir("write");
        let path = dir.join("covers").join("x.png");
        write_cover(&path, b"png").expect("escribir");
        assert_eq!(std::fs::read(&path).unwrap(), b"png");
        let _ = std::fs::remove_dir_all(dir);
    }
}
