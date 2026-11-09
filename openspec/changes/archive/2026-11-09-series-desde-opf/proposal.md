## Why

Muchas bibliotecas personales tienen sagas y colecciones. Candil indexaba título, autor e idioma, pero descartaba `calibre:series` / `belongs-to-collection`, así que los libros de una serie quedaban sueltos y sin un orden útil.

## What Changes

- El OPF se lee también para **serie** y **posición**: variantes Calibre (`calibre:series`, `calibre:series_index`) y EPUB3 (`belongs-to-collection` + `collection-type=series` + `group-position`).
- `Book` y `BookEntry` guardan `series` y `series_index`; `library.json` los conserva con defaults.
- Un `meta_version` por entrada fuerza a releer metadatos una vez en el próximo escaneo, sin migración manual.
- La tarjeta muestra la serie y el número debajo del autor.
- Nuevo orden **Serie** en la biblioteca: agrupa por nombre y posición, con títulos sin serie al final.
- Tests con `mini.epub` (Calibre) y `notes.epub` (EPUB3).

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: la tarjeta y el orden incorporan series/colecciones.

## Scope

**Incluido:** lectura de serie/posición, persistencia, relectura única, etiqueta en la tarjeta, orden por serie.

**No incluido:** filtro por serie, cabeceras de grupo en la grilla, colecciones creadas por el usuario, sinopsis/volúmenes, detección de duplicados.

## Acceptance Criteria

- [x] Un EPUB con metadatos Calibre expone serie y posición.
- [x] Un EPUB3 con `belongs-to-collection` y `collection-type=series` expone serie y posición.
- [x] Las entradas existentes sin `meta_version` se releen una vez y conservan progreso, ajustes, portada y fechas.
- [x] La tarjeta muestra `Serie · N` cuando existe; sin serie no agrega una línea vacía visible.
- [x] El orden **Serie** agrupa por nombre y posición; los libros sin serie van al final.
- [x] `cargo test`, `fmt`, `clippy -D warnings`, `scripts/check.sh` y `dist < 8 MB` pasan.

## Impact

- `src/epub.rs`, `src/library.rs`, `src/app.rs`.
- `scripts/make_fixture.py`, `tests/fixtures/mini.epub`, `tests/fixtures/notes.epub`, `tests/epub_smoke.rs`.
- `docs/adr/0020-series-opf.md`, `README.md`, `docs/HANDOFF.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md`.
- Capturas de biblioteca regeneradas por la altura de tarjeta.
