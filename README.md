# lector

Lector de EPUB minimalista en Rust. Una columna, scroll continuo, tipografía
cuidada y cero dependencias del sistema para dibujar el texto: egui + un
traductor propio de XHTML a bloques.

Binario final (`dist`): **5.1 MB**, arranque instantáneo, sin webview ni motor HTML.

## Uso

```bash
cargo run --profile fast -- /ruta/libro.epub   # desarrollo: ~1 s por cambio
cargo run --release -- /ruta/libro.epub        # uso diario
cargo run --profile dist -- /ruta/libro.epub   # artefacto final, más chico
cargo run --profile fast                       # abre el último libro o el inicio
```

También se puede arrastrar un `.epub` sobre la ventana.

### Perfiles de compilación

`lto` es la razón por la que un cambio de una línea tardaba 26 s: obliga a
re-enlazar los 249 crates en cada build. Tres perfiles para tres momentos:

| perfil | cuándo | cambio de una línea | binario |
|---|---|---|---|
| `fast` | iterar | ~1.3 s (incremental) | 9.9 MB |
| `release` | uso diario | ~17 s (LTO thin) | 7.7 MB |
| `dist` | distribuir | ~90 s (LTO fat, `opt-level="z"`) | 5.1 MB |

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
| `Esc` | cerrar panel |
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
  main.rs    arranque, flags de desarrollo, modo --inspect
  app.rs     ventana, barra, scroll, paneles, entrada
  epub.rs    zip → container.xml → OPF → spine + índice (nav.xhtml / toc.ncx)
  xhtml.rs   XHTML/HTML → Vec<Block> (modelo intermedio, sin CSS)
  theme.rs   presets de color
  config.rs  preferencias y progreso (JSON)
  fonts.rs   carga de familias en egui
```

El modelo intermedio es plano: cada capítulo es una lista de `Block`
(párrafo, título, cita, ítem de lista, regla, imagen) y cada bloque una lista de
`Run` con estilo inline. Nada de DOM ni de motor CSS: el lector decide la
apariencia, el EPUB solo aporta estructura.

Dos parsers: el estricto (roxmltree, XHTML válido) y un parser tolerante propio
para los EPUB que traen `text/html` mal formado — caso real y frecuente.

## Estado

Probado contra 44 EPUB reales (Calibre, Sigil, OCR, Anna's Archive): todos
abren, todas las imágenes referenciadas resuelven. `cargo test` cubre el parser,
la resolución de rutas y el saneamiento de entidades.

## Limitaciones conocidas

- CSS del libro ignorado por diseño (los temas mandan).
- Índice a nivel de archivo, no de ancla: libros con todo el texto en un solo
  XHTML y anclas internas colapsan en una entrada.
- Sin paginado por columnas ni sinónimos/hyphenation.
- Sin búsqueda ni anotaciones.

## Roadmap

- **M2 — Biblioteca:** estantería con metadatos, portadas, orden y filtros.
- **M3 — Anclas:** índice por fragmento, saltos dentro del capítulo.
- **M4 — Extras:** búsqueda en el libro, marcadores, exportar citas.

## Modo diagnóstico

```bash
cargo run --release -- --inspect /ruta/libro.epub     # metadatos, spine, bloques, imágenes
cargo run --release -- --shot out.png --chapter 13 libro.epub   # captura sin interacción
cargo run --release -- --shot out.png --panel settings libro.epub
```

## Licencia de las fuentes

Literata © TypeTogether, SIL Open Font License 1.1 (`assets/fonts/Literata-OFL.txt`).
