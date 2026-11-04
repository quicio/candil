---
id: spec-20
title: Biblioteca — índice, estantería y apertura
status: draft
version: 1.0.0
updated: 2026-11-03
milestone: M2
depends_on: [adr-0007, adr-0008, spec-10, spec-21]
blocks: []
---

# Spec 20 — Biblioteca (M2)

> **Estado: draft.** Requiere revisión de Hugo antes de implementar.
> Tareas diseñadas para que un agente las ejecute sin contexto previo.

## Objetivo

Dejar de abrir un archivo por vez: una pantalla de biblioteca que indexa carpetas
del disco, muestra portadas con título/autor/progreso, permite ordenar, filtrar,
buscar y abrir en un click, y recuerda todo entre arranques.

## Alcance

Entra: índice persistente, escaneo de carpetas, portadas cacheadas (spec 21),
grilla, orden, filtro, búsqueda incremental, apertura y vuelta, migración del
progreso existente, estados vacíos y de error.

**No entra:** índices por ancla, paginado, notas, edición de metadatos, estantes
manuales, watcher del sistema, sincronización, copia de archivos.

## Modelo de datos (ADR-0007)

```rust
// src/library.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct BookEntry {
    pub path: String,          // ruta absoluta canónica, también es la clave
    pub title: String,
    pub author: String,
    pub lang: String,
    pub cover: Option<String>, // ruta relativa dentro del dir de datos: "covers/<hash>.png"
    pub size: u64,
    pub mtime: i64,            // segundos
    pub progress: Progress,    // reusa config::Progress
    pub added: i64,
    pub last_opened: Option<i64>,
    pub missing: bool,         // el archivo ya no está
    pub error: Option<String>, // el libro no se pudo leer al importar
}

#[derive(Serialize, Deserialize, Default)]
pub struct Library {
    pub folders: Vec<String>,          // carpetas vigiladas
    pub books: Vec<BookEntry>,
}
```

Rutas de datos: `dirs::config_dir()/lector/{library.json, covers/}`.

## UI

```
┌─────────────────────────────────────────────────────────┐
│ ≡  Aa   [buscar…            ]        ⌄orden  ▸filtrar   │  barra
├─────────────────────────────────────────────────────────┤
│  ▛▀▀▜  ▛▀▀▜  ▛▀▀▜  ▛▀▀▜  ▛▀▀▜                           │
│  ███   ███   ███   ███   ███   ← portadas 160×240       │
│  Título   Título    …                                   │
│  Autor    Autor                                         │
│  ▓▓▓░ 42%  —                                           │
└─────────────────────────────────────────────────────────┘
```

- `Esc` en el lector vuelve a la biblioteca; `Esc` en la biblioteca sin overlay no
  hace nada.
- Click en una portada abre el libro. Click derecho: "Mostrar en Finder",
  "Quitar de la biblioteca" (no borra el archivo).
- Barra superior: buscador (título, autor, nombre de archivo), orden
  (`Reciente / Título / Autor / Añadido`), filtro (`Todos / Leyendo / Sin empezar /
  Terminados / Faltantes`).
- Estados vacíos: sin carpetas ("Añadí una carpeta"), carpeta sin EPUB ("No hay
  libros acá"), búsqueda sin resultados ("Nada coincide con «x»").
- Progreso a la vista: barra fina bajo el título + %.

## Tareas

### T1 — Modelo y persistencia de la biblioteca

**Archivos:** crear `src/library.rs`; modificar `src/lib.rs` (agregar `pub mod library;`).

**Pasos**
1. Escribir tests primero en `#[cfg(test)] mod tests` de `library.rs`:
   - `Library::default().save_to(&dir)` y `load_from(&dir)` devuelven lo mismo.
   - JSON sin campos nuevos (`progress`, `missing`) se carga sin error (`serde(default)`).
   - JSON corrupto → `Library::default()`, sin panic.
2. Implementar `Library { folders, books }` con `load_from(dir) / save_to(dir)`,
   `data_dir() -> PathBuf` (usa `dirs::config_dir()/lector`), y `upsert(entry)`
   que reemplaza por `path` o agrega.
3. `cargo test library` verde.

**Criterio:** los tres tests pasan; `library.json` escrito es legible a ojo.

### T2 — Extracción de metadatos de un book

**Archivos:** `src/library.rs`.

**Pasos**
1. Test: importar `tests/fixtures/mini.epub` produce
   `title == "Libro de prueba"`, `author == "Autora Ficticia"`, `lang == "es"`,
   `size > 0`, `error.is_none()`.
2. Implementar `BookEntry::from_path(path) -> BookEntry` usando
   `epub::Book::open`; `mtime` desde `std::fs::metadata`
   (`duration_since(UNIX_EPOCH)`), `path` canonicalizado.
3. Si `Book::open` falla: entrada con `error = Some(msg)`, título = nombre del
   archivo. Nunca propagar el error al escaneo.

**Criterio:** un `.epub` corrupto agrega entrada con `error` y la estantería sigue
funcionando.

### T3 — Escanear carpetas

**Archivos:** `src/library.rs`.

**Pasos**
1. Test con `tests/fixtures/`: escanear la carpeta devuelve 1 entrada (el fixture);
   escanear una carpeta inexistente devuelve 0 y no entra en pánico.
2. Implementar `Library::scan_folder(&mut self, dir) -> ScanStats { found, added,
   updated, unchanged, failed }`:
   - Recorrido recursivo con `std::fs::read_dir`, sin seguir symlinks.
   - Filtro por extensión `.epub` (case-insensitive).
   - Incremental: si `path` existe en `books` y `(size, mtime)` coinciden, cuenta
     `unchanged` y saltea el `Book::open`.
3. `folders` recuerda la carpeta; `rescan()` recorre todas.

**Criterio:** 50 libros escaneados en ≤ 5 s; segundo escaneo no reabre zips.

### T4 — Portadas (detalle en spec 21)

**Archivos:** `src/library.rs`, `src/covers.rs` (nuevo).

**Pasos**
1. Tests de resolución en cascada: `cover-image` (EPUB3), `meta name="cover"`
   (EPUB2), primer `<img>` del spine; y el caso "sin portada" → `None`.
2. `extract_cover(book: &mut Book) -> Option<Vec<u8>>` devuelve los bytes de la
   imagen declarada.
3. `thumbnail(bytes, height: 320) -> ColorImage/png` y escritura en
   `covers/<sha-ish>.png` (hash simple de la ruta: `DefaultHasher`).
4. Idempotencia: si el PNG existe y `mtime` no cambió, no se regenera.

**Criterio:** ≥ 90% de los libros del corpus con portada declarada muestran
portada; `covers/` < 100 KB por libro.

### T5 — Pantalla de biblioteca

**Archivos:** `src/app.rs`.

**Pasos**
1. Agregar `enum Screen { Library, Reader }` en `Lector`; arrancar en `Library`
   si `library.books` no está vacío, si no en `Library` vacía (nunca abrir un libro
   solo porque fue el último: comportamiento `--resume` es un flag aparte).
2. `fn library_screen(&mut self, ctx, th)`: barra superior + grilla responsive con
   `egui::Grid` o cálculo de columnas por ancho (`ancho / 180`).
3. Cada celda: textura de portada (`cover` si existe; si no, marcador con iniciales
   sobre `th.panel`, y `error` en rojo si `entry.error.is_some()`), título (2 líneas
   máximo, `truncate`), autor, barra de progreso fina.
4. Click → `self.open_book(path)` (reusar el `open` actual) y `screen = Reader`.

**Criterio:** captura `--shot` con la estantería poblada; click abre el libro.

### T6 — Orden, filtro y búsqueda

**Archivos:** `src/app.rs` (+ tests puros en `src/library.rs`).

**Pasos**
1. Implementar `Library::view(&self, orden, filtro, query) -> Vec<&BookEntry>` con
   tests puros (sin egui): orden por título usa `to_lowercase` y normaliza acentos
   (`collate`), filtro `Leyendo` = `progress.chapter > 0 || offset > 0`,
   `Terminados` = offset ≥ 95% del capítulo? **no**: `Terminados` requiere campo
   `finished` (ver "Riesgos"); en M2 `Terminados` queda fuera y el filtro es
   `Todos / Leyendo / Sin empezar / Faltantes`.
2. UI: `TextEdit` de búsqueda (focus con `⌘F`), `ComboBox` de orden, botones de
   filtro.
3. Búsqueda incremental sobre título, autor y nombre de archivo, case-insensitive.

**Criterio:** 4 tests puros de `view()` + captura con búsqueda activa.

### T7 — Navegación entre biblioteca y lector

**Archivos:** `src/app.rs`.

**Pasos**
1. `Esc` en `Reader` cierra overlays; si no hay overlay, vuelve a `Library`
   (guardando progreso antes).
2. `Esc` en `Library` no hace nada.
3. Marcar `last_opened` al abrir y reordenar por reciente.
4. Título de ventana: `lector — <biblioteca>` o `lector — <título del libro>`.

**Criterio:** abrir → leer → `Esc` vuelve a la grilla con el progreso actualizado
en la tarjeta.

### T8 — Migración del progreso existente

**Archivos:** `src/library.rs`, `src/config.rs`.

**Pasos**
1. Al cargar la biblioteca por primera vez (si `library.json` no existe y
   `config.json` tiene `progress`), volcar cada `{chapter, offset}` a la entrada
   cuyo `path` coincida.
2. Dejar de escribir `progress` en `config.json` (migrar la lectura), manteniendo
   el campo en la struct con `#[serde(default)]` por compatibilidad de archivos
   viejos; documentarlo en ADR-0005 como enmienda.
3. Test: config con progreso + biblioteca vacía → tras migrar, la entrada tiene
   ese progreso y `config.progress` se vacía.

**Criterio:** abrir la app tras actualizar no pierde la posición de lectura.

### T9 — Carpetas vigiladas y estados

**Archivos:** `src/app.rs`, `src/library.rs`.

**Pasos**
1. Botón "Añadir carpeta…" (`rfd::FileDialog::pick_folder`), "Refrescar" (rescan),
   y por carpeta un "×" para quitarla (no borra libros del disco, solo deja de
   vigilarla).
2. Estados vacíos con texto accionable (ver UI arriba).
3. Errores de importación visibles: contador "3 libros no se pudieron leer" con
   tooltip por libro.

**Criterio:** quitar una carpeta no rompe entradas existentes; refrescar informa
cuántos se agregaron/actualizaron.

### T10 — Verificación visual reproducible

**Archivos:** `src/main.rs`, `src/app.rs`.

**Pasos**
1. Extender `DevOpts` con `screen: Option<String>` (`"library"`) y
   `library_path: Option<PathBuf>` (para usar una biblioteca de prueba sin tocar
   la real).
2. `--shot` debe funcionar en ambas pantallas.
3. Generar 4 capturas en `docs/img/`: biblioteca vacía, poblada, con búsqueda,
   con error de importación. Referenciarlas en el README.

**Criterio:** `--shot --screen library` produce PNG con la grilla real.

### T11 — Cierre del milestone

**Pasos**
1. `cargo test` + `cargo fmt --check` limpios.
2. Medir: tiempo de escaneo de 50 libros, tamaño de `covers/`, tamaño del binario
   `dist` (objetivo < 8 MB).
3. Actualizar `ROADMAP.md` (M2 → hecho), esta spec (`status: implementada`) y
   `HANDOFF.md` con las métricas nuevas.
4. Actualizar ADR-0007/0008 de `propuesta` a `aceptada` si se implementaron tal cual.

**Criterio:** los 5 puntos del criterio de salida de M2 en `ROADMAP.md` se pueden
mostrar con un comando cada uno.

## Riesgos y decisiones abiertas

| riesgo | mitigación |
|---|---|
| Rutas con acentos/emojis | usar `String` de `PathBuf::to_string_lossy`, nunca bytes crudos; test con nombre acentuado |
| Libros en carpetas de red lentas | escaneo en el hilo de UI en M2 (aceptado); si molesta, mover a hilo con progreso |
| `Terminados` necesita saber el final del libro | pospuesto: requiere % real por capítulo; hoy solo hay offset dentro del capítulo |
| Duplicados (mismo libro en dos carpetas) | clave = ruta canónica; se muestran dos veces. Deduplicar por título+autor queda para M3 |
| Portada enorme (mapa) elegida como portada | el cascade toma lo declarado; si el resultado tiene aspecto > 2:1 se ignora (ver spec 21) |

## Criterio de salida (repetido de ROADMAP)

1. 50 EPUB escaneados en ≤ 5 s con caché.
2. ≥ 90% de portadas para libros que las declaran.
3. Reabrir la app no reescanea.
4. `cargo test` verde + 4 capturas.
5. `--inspect` sin regresiones sobre el corpus.
