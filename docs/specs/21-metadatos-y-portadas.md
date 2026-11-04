---
id: spec-21
title: Metadatos y portadas
status: draft
version: 1.0.0
updated: 2026-11-03
milestone: M2
depends_on: [adr-0008, spec-20]
---

# Spec 21 — Metadatos y portadas

Detalle de la tarea T4 de la spec 20. Se puede implementar y testear sin UI.

## Resolución de la portada

Cascada, se corta en la primera que exista **y** decodifique:

1. `manifest item` con `properties` que contenga `cover-image` (EPUB3).
2. `metadata meta name="cover"` → su `content` es un `id` del manifest (EPUB2/Calibre).
3. `meta property="cover"` / `name="cover"` apuntando a una ruta.
4. Primer `img` del primer documento del spine que tenga uno.

Validaciones antes de aceptar:

- Los bytes decodifican con `image::load_from_memory`.
- Relación de aspecto resultante ≤ 2.0 de alto/ancho (descarta mapas y banners
  apaisados que algunos libros declaran como portada).
- Si falla cualquiera, se sigue con el siguiente candidato de la cascada.

## Derivados

- Miniatura: **alto 320 px**, ancho proporcional (`image::imageops::resize`,
  filtro `Lanczos3`), guardada como PNG en `covers/<hash>.png`.
- `hash` = `DefaultHasher` sobre la ruta canónica del libro, formateado `{:016x}`.
  Determinista entre arranques (misma versión de Rust) y sin dependencias.
- Si el PNG existe y `mtime` del libro no cambió: no se regenera.
- Si el libro cambia (`mtime` distinto): se regenera y se sobrescribe.

## Metadatos extendidos

Se extraen al importar, se guardan en `BookEntry` (opcionales, no rompen el JSON):

| campo | fuente |
|---|---|
| `title` | `dc:title`; fallback nombre de archivo |
| `author` | `dc:creator` (el primero si hay varios) |
| `lang` | `dc:language` |
| `series` | `meta name="calibre:series"` |
| `series_index` | `meta name="calibre:series_index"` |
| `publisher` | `dc:publisher` |

`series`/`series_index` se muestran en el tooltip de la tarjeta; agrupar por serie
es M3.

## Tareas

### T4.1 — Acceso a la portada declarada

**Archivos:** `src/epub.rs` (agregar método), `src/library.rs` (tests).

**Pasos**
1. Test sobre `tests/fixtures/mini.epub`: no declara portada → `cover_bytes()`
   devuelve el primer `<img>` del capítulo 1 (`OEBPS/Images/punto.png`).
2. Crear un segundo fixture con portada declarada por `properties="cover-image"`
   (`scripts/make_fixture.py` gana un flag `--with-cover`) y testear que la
   devuelve.
3. Implementar en `Book`: `cover_bytes(&mut self) -> Option<(Vec<u8>, String)>`
   con la cascada 1→4 y las validaciones.

**Criterio:** dos tests verdes (con y sin portada declarada).

### T4.2 — Miniatura y caché

**Archivos:** `src/covers.rs` (nuevo, `pub mod covers;` en `lib.rs`).

**Pasos**
1. Test: `thumbnail(png_2x2, 320)` devuelve imagen con alto 320 y ancho ≥ 1.
2. Test: `cover_path_for(dir, "/libro.epub")` es estable entre llamadas.
3. Implementar `thumbnail(&[u8], height) -> Option<Vec<u8>>` (PNG de salida) y
   `write_cover(dir, hash, png) -> io::Result<()>`.
4. Test de idempotencia con archivo temporal: segunda llamada no reescribe
   (`mtime` del PNG igual).

**Criterio:** miniatura correcta, ruta estable, sin reescrituras innecesarias.

### T4.3 — Conectar con el escaneo

**Archivos:** `src/library.rs`.

**Pasos**
1. En `BookEntry::from_path`, tras abrir el libro, generar miniatura si falta.
2. Si la portada falla, `cover = None` y la UI dibuja el marcador tipográfico.
3. Test: escanear el fixture produce `cover.is_some()`; escanear un EPUB sin
   imágenes produce `cover.is_none()` sin error.

**Criterio:** estantería con portadas reales y marcador donde no hay.

## Mediciones a reportar

- % de libros con portada sobre el corpus (objetivo ≥ 90% de los que declaran).
- Tamaño medio y total de `covers/`.
- Tiempo de escaneo con y sin caché de portadas.
