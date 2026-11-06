# lector

Lector de EPUB minimalista en Rust. Una columna, scroll continuo, tipografía
cuidada y cero dependencias del sistema para dibujar el texto: egui + un
traductor propio de XHTML a bloques. Incluye una biblioteca que indexa carpetas
sin copiar los libros.

Binario final (`dist`): **5.2 MiB** (5.499.488 B), arranque instantáneo, sin
webview ni motor HTML.

## Documentación

| documento | contenido |
|---|---|
| [`AGENTS.md`](AGENTS.md) | comandos y reglas duras para agentes |
| [`docs/HANDOFF.md`](docs/HANDOFF.md) | estado, invariantes, verificación sin ojos |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | milestones M1–M4 con criterios de salida |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | módulos, modelo `Block`, cómo extenderlo |
| [`docs/specs/`](docs/specs/) | especificaciones por milestone |
| [`docs/adr/`](docs/adr/) | decisiones con contexto y alternativas |

## Uso

```bash
cargo run --profile fast -- /ruta/libro.epub   # desarrollo: ~3-6 s por cambio
cargo run --release -- /ruta/libro.epub        # uso diario
cargo run --profile dist -- /ruta/libro.epub   # artefacto final, más chico
cargo run --profile fast                       # abre la biblioteca
cargo run --profile fast -- --resume           # reabre el último libro
cargo run --profile fast -- --scan ~/Libros    # indexa una carpeta sin abrir la UI
scripts/check.sh                               # fmt + clippy + tests + inspect
```

También se puede arrastrar un `.epub` sobre la ventana. La primera vez, usá
“Añadir carpeta…” en la biblioteca para indexar una carpeta con EPUB.

### Perfiles de compilación

`lto` es la razón por la que un cambio de una línea tardaba 26 s: obliga a
re-enlazar los 249 crates en cada build. Tres perfiles para tres momentos:

| perfil | cuándo | cambio de una línea | binario |
|---|---|---|---|
| `fast` | iterar | ~3–6 s (incremental, 2026-11-05) | 10.1 MB |
| `release` | uso diario | ~17 s (LTO thin) | 7.7 MB |
| `dist` | distribuir | ~56 s (LTO fat, `opt-level="z"`) | 5.5 MB |

### Teclas

| tecla | acción |
|---|---|
| `Space` / `PgDn` / `↓` | avanzar |
| `PgUp` / `↑` | retroceder |
| `←` / `→` | capítulo anterior / siguiente |
| `Home` | volver al inicio del capítulo |
| `+` / `-` | tamaño de fuente |
| `T` | índice |
| `S` | tipografía |
| `Esc` | cerrar panel; en el lector, volver a la biblioteca |
| `⌘F` | buscar en la biblioteca |
| `⌘O` | abrir archivo |

## Temas y tipografía

Cinco presets prearmados (Papel, Noche, Claro, Ámbar, Contraste) y ajustes de
tamaño, interlineado, ancho de columna y espacio entre párrafos. Todo se guarda
en `~/Library/Application Support/lector/config.json` junto con el progreso de
lectura por libro.

Tipografía: Literata (OFL) embebida en cuatro variantes — regular, itálica,
bold, bold-itálica — más semibold para títulos, instanciadas desde la fuente
variable con `fonttools`.

## Arquitectura

```
src/
  main.rs    arranque, flags de desarrollo, modo --inspect / --scan
  app.rs     biblioteca, lector, barra, scroll, paneles, entrada
  epub.rs    zip → container.xml → OPF → spine + índice + portada declarada
  xhtml.rs   XHTML/HTML → Vec<Block> (entidades, whitespace, pre, tablas, SVG raster)
  library.rs índice de carpetas, escaneo incremental y BookEntry
  covers.rs  miniaturas de portada, hash FNV y caché
  store.rs   JSON atómico y backup de archivos corruptos
  theme.rs   presets de color
  config.rs  preferencias y progreso (JSON atómico)
  fonts.rs   carga de familias en egui
```

El modelo intermedio es plano: cada capítulo es una lista de `Block`
(párrafo, título, cita, ítem de lista, regla, imagen) y cada bloque una lista de
`Run` con estilo inline. Nada de DOM ni de motor CSS: el lector decide la
apariencia, el EPUB solo aporta estructura.

Dos parsers: el estricto (roxmltree, XHTML válido) y un parser tolerante propio
para los EPUB que traen `text/html` mal formado — caso real y frecuente. Ambos
comparten el saneamiento de entidades, la regla de whitespace, `<pre>` y el
mapeo de `<svg><image>` a `Kind::Image`.

## Estado

Probado contra 43 EPUB reales (Calibre, Sigil, OCR, Anna's Archive, 2026-11-06):
todos abren, **0 capítulos vacíos** y **2279/2279 imágenes resueltas**, incluidas
las portadas/mapas que vienen envueltos en `<svg><image>`. `cargo test` corre 38
tests unitarios y 8 de integración sobre fixtures propios versionados
(`mini.epub` y `edge.epub`, regenerables con `python3 scripts/make_fixture.py`),
sin depender de libros externos.

La biblioteca (M2, en curso) indexa `~/Libros` en **1.57 s** la primera vez y
**0.02 s** con caché; **41/43 portadas (95,3 %)** y **4,2 MB** en `covers/`.
Pantalla con grilla, búsqueda, orden, filtros y vuelta con `Esc`.

## Limitaciones conocidas

- CSS del libro ignorado por diseño (los temas mandan).
- Índice a nivel de archivo, no de ancla: libros con todo el texto en un solo
  XHTML y anclas internas colapsan en una entrada.
- Sin paginado por columnas ni sinónimos/hyphenation.
- Sin búsqueda dentro del libro ni anotaciones.
- Tablas y `<dl>` se leen como bloques apilados; el SVG vectorial (paths) no se
  renderiza, solo el raster embebido en `<svg><image>`.
- La biblioteca todavía no migra el progreso viejo a `library.json`, no permite
  quitar carpetas y no tiene watcher: se refresca a mano.
- El filtro “Terminados” y el progreso por fracción llegan con M3.

## Roadmap

- **M2 — Biblioteca (en curso):** índice, portadas y pantalla hechos; faltan
  migración de progreso, gestión de carpetas y cierre visual.
- **M3 — Anclas:** índice por fragmento, saltos dentro del capítulo.
- **M4 — Extras:** búsqueda en el libro, marcadores, exportar citas.

## Modo diagnóstico

```bash
cargo run --release -- --inspect /ruta/libro.epub     # metadatos, spine, bloques, imágenes
cargo run --release -- --inspect tests/fixtures/edge.epub   # bordes del parser
cargo run --release -- --scan ~/Libros                # índice + portadas sin UI
cargo run --release -- --shot out.png                 # captura de la biblioteca
cargo run --release -- --shot out.png --chapter 13 libro.epub   # captura del lector
cargo run --release -- --shot out.png --panel settings libro.epub
```

## Licencia de las fuentes

Literata © TypeTogether, SIL Open Font License 1.1 (`assets/fonts/Literata-OFL.txt`).
