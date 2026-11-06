---
id: spec-21
title: Metadatos y portadas
status: implementada
version: 1.2.0
updated: 2026-11-06
milestone: M2
depends_on: [adr-0008, spec-20]
---

# Spec 21 — Metadatos y portadas

Detalle de la tarea T4 de la spec 20. Se puede implementar y testear sin UI.

> Implementada el 2026-11-06. Resultados sobre `~/Libros`: **41/43 portadas
> (95,3 %)**, **4,2 MB** en `covers/`, primera extracción incluida en un escaneo
> de 1,57 s. `series`/`publisher` quedan para M3.

## Resolución de la portada

Cascada, se corta en la primera que exista **y** decodifique:

1. `manifest item` con `properties` que contenga `cover-image` (EPUB3).
2. `metadata meta name="cover"` → su `content` es un `id` del manifest (EPUB2/Calibre).
3. `meta property="cover"` / `name="cover"` apuntando a una ruta.
4. Primer `<img>` (o `<svg><image>`) del primer documento del spine que tenga uno.

Validaciones antes de aceptar:

- Los bytes decodifican con `image::load_from_memory`.
- Relación de aspecto resultante ≤ 2.0 de ancho/alto (descarta mapas y banners
  apaisados que algunos libros declaran como portada).
- Si falla cualquiera, se sigue con el siguiente candidato de la cascada.

## Derivados

- Miniatura: **alto 320 px como máximo**, ancho proporcional (`image::imageops`,
  filtro `Lanczos3`), guardada como PNG en `covers/<hash>.png`. **No se agranda**
  una imagen más chica que 320.
- `hash` = FNV-1a de 64 bits sobre la ruta canónica del libro, formateado
  `{:016x}`. Es estable entre versiones de Rust (a diferencia de `DefaultHasher`).
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
1. Test sobre `tests/fixtures/mini.epub`: declara `properties="cover-image"` →
   `cover_bytes()` devuelve `OEBPS/Images/punto.png`. Test equivalente sobre
   `edge.epub` para el fallback `<svg><image>`.
2. El segundo fixture es `edge.epub` (fallback SVG); no hace falta un tercero.
3. Implementado en `Book`: `cover_bytes(&mut self) -> Option<Vec<u8>>` con la
   cascada 1→4, validación de decodificación y aspecto.

**Criterio:** tests verdes para portada declarada y fallback SVG.

### T4.2 — Miniatura y caché

**Archivos:** `src/covers.rs` (nuevo, `pub mod covers;` en `lib.rs`).

**Pasos**
1. Test: una imagen de 10×20 no se agranda; una de 400×600 se reduce a alto 320.
2. Test: `cover_path_for(dir, "/libro.epub")` es estable entre llamadas y cambia
   con otra ruta.
3. Implementado `thumbnail(&[u8], height) -> Option<Vec<u8>>` (PNG de salida) y
   `write_cover(path, png) -> io::Result<()>`.
4. La idempotencia la resuelve el escaneo: si `size`/`mtime` no cambian, ni
   siquiera se abre el EPUB.

**Criterio:** miniatura correcta, sin upscaling, ruta estable, sin reescrituras
innecesarias.

### T4.3 — Conectar con el escaneo

**Archivos:** `src/library.rs`.

**Pasos**
1. `BookEntry::from_path` extrae metadatos; `from_path_with_cover` recibe el
   directorio de `covers/` inyectable y genera la miniatura.
2. Si la portada falla, `cover = None` y la UI dibuja el marcador tipográfico.
3. Tests: escanear los fixtures produce `cover.is_some()` y una portada por
   libro; un EPUB corrupto no propaga el error.

**Criterio:** estantería con portadas reales y marcador donde no hay.

## Mediciones (2026-11-06)

| métrica | resultado |
|---|---|
| portadas sobre `~/Libros` | **41/43 (95,3 %)** |
| libros que declaran portada | 39 + 2 fallback; 2 sin candidata |
| tamaño total de `covers/` | **4,2 MB** (~98 KB/libro) |
| escaneo primera vez | **1,57 s** (43 EPUB, metadatos + portadas) |
| escaneo con caché | **0.02 s** (43 sin cambios) |
| miniaturas | alto máximo 320, sin upscaling |
