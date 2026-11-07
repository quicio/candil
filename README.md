# Candil

**Leer despacio, sin ruido.**

Lector de EPUB minimalista en Rust. Una columna, scroll continuo, tipografía
cuidada y cero dependencias del sistema para dibujar el texto: egui + un
traductor propio de XHTML a bloques. Incluye una biblioteca que indexa carpetas
sin copiar los libros, anclas internas, progreso estable por bloque,
destacados/notas —con colores, índice global y export Markdown—, búsqueda dentro
del libro, salto por porcentaje/capítulo, diccionario/traducción y análisis con IA.

Binario final (`dist`): **6.4 MiB** (6.747.328 B), arranque instantáneo, sin
webview ni motor HTML.

## Documentación

| documento | contenido |
|---|---|
| [`AGENTS.md`](AGENTS.md) | comandos y reglas duras para agentes |
| [`docs/HANDOFF.md`](docs/HANDOFF.md) | estado, invariantes, verificación sin ojos |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | milestones M1–M6 con criterios de salida |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | módulos, modelo `Block`, cómo extenderlo |
| [`docs/BUILD.md`](docs/BUILD.md) | compilar en macOS, Linux y Windows; cross-compiling y CI |
| [`docs/specs/`](docs/specs/) | especificaciones por milestone |
| [`docs/adr/`](docs/adr/) | decisiones con contexto y alternativas |
| [`openspec/`](openspec/) | cambios y specs de OpenSpec; cambio activo `consolidar-ux-ui` |

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

### Compilar en otros sistemas

La vía recomendada es compilar nativo en cada SO. Guía completa:
[`docs/BUILD.md`](docs/BUILD.md).

```bash
# macOS (Xcode Command Line Tools)
cargo build --profile dist

# Ubuntu/Debian (X11/Wayland/GL)
sudo apt install build-essential pkg-config libx11-dev libxi-dev \
  libxcursor-dev libxrandr-dev libxkbcommon-dev libwayland-dev \
  libgl1-mesa-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
cargo build --profile dist

# Windows (Rust MSVC + VS Build Tools "Desktop development with C++")
cargo build --profile dist
```

El binario queda en `target/dist/candil` (o `candil.exe`). La matriz
`.github/workflows/build.yml` compila y sube artefactos en macOS, Linux y
Windows. Para build universal de macOS hay instrucciones con `lipo` en la guía.

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
| `D` | destacados y notas (por libro o global) |
| `G` | ir a porcentaje/capítulo |
| `K` | consultar selección (diccionario/traducción/IA) |
| `⌘,` | configuración |
| `H` | destacar la selección |
| `N` | nota sobre la selección |
| `Esc` | cerrar panel / salir de la selección; en el lector, volver a la biblioteca |
| `⌘F` | buscar en la biblioteca / en el libro abierto |
| `Enter` (búsqueda) | ir al resultado |
| `n` / `N` (búsqueda) | siguiente / anterior resultado |
| `⌘O` | abrir archivo |
| `F1` / `?` | ayuda de atajos |

## Temas y tipografía

Cinco presets prearmados (Papel, Noche, Claro, Ámbar, Contraste) y ajustes de
tamaño, interlineado, ancho de columna y espacio entre párrafos. Los ajustes se
guardan en `~/Library/Application Support/candil/config.json`; el progreso de los
libros indexados vive en `library.json`, junto a las portadas. Con el checkbox
“Solo para este libro” del panel `Aa` se fija tema/tipografía por libro; el
indicador `Aa·` avisa que ese libro tiene ajustes propios.

Tipografía: Literata (OFL) embebida en cuatro variantes — regular, itálica,
bold, bold-itálica — más semibold para títulos, instanciadas desde la fuente
variable con `fonttools`.

## Arquitectura

```
src/
  main.rs    arranque, flags de desarrollo, modo --inspect / --scan
  app.rs     biblioteca, lector, barra, scroll, paneles, entrada
  epub.rs    zip → container.xml → OPF → spine + índice con #ancla + portada
  xhtml.rs   XHTML/HTML → Vec<Block> (entidades, whitespace, pre, tablas, SVG, anclas)
  library.rs índice de carpetas, escaneo incremental y BookEntry
  annotations.rs destacados/notas por libro (offsets de texto + cita)
  search.rs  índice efímero de búsqueda (normaliza sin cambiar longitud)
  consulta.rs diccionario/traducción/IA (HTTP ureq) e historial local
  covers.rs  miniaturas de portada, hash FNV y caché
  store.rs   JSON atómico y backup de archivos corruptos
  theme.rs   presets de color
  config.rs  preferencias en config.json (progreso migrado a library.json)
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

Probado contra 43 EPUB reales de distintas procedencias (Calibre, Sigil, OCR;
2026-11-06):
todos abren, **0 capítulos vacíos** y **2279/2279 imágenes resueltas**, incluidas
las portadas/mapas que vienen envueltos en `<svg><image>`. `cargo test` corre 98
tests unitarios (1 live ignorado) y 9 de integración sobre fixtures propios versionados
(`mini.epub` y `edge.epub`, regenerables con `python3 scripts/make_fixture.py`),
sin depender de libros externos.

La biblioteca (M2, cerrada) indexa `~/Libros` en **1.57 s** la primera vez y
**0.02 s** con caché; en la medición de 43 EPUB mostró **41 portadas (95,3 %)** y
**4,2 MB** en `covers/`. El cierre con **50 EPUB** (43 reales + 7 copias) dio
**1.65 s**, **48/50 portadas (96 %)** y **5,4 MB**. Hoy `~/Libros` tiene 40 libros
y los 40 con portada. Pantalla con grilla, búsqueda, orden, filtros, gestión de
carpetas y vuelta con `Esc`.

M3 (lectura fina) también está cerrada: el índice salta por `#ancla` dentro y
entre capítulos, los enlaces internos son clicables y la posición se guarda como
`chapter + ratio + bloque + fracción del bloque`, así que cambiar fuente o
ventana no descoloca la lectura. Tema y tipografía se pueden fijar **solo para un
libro** con el checkbox del panel `Aa`; el indicador `Aa·` avisa que ese libro
tiene ajustes propios.

M4 (extras) está cerrado con destacados/notas y búsqueda dentro del libro.
Seleccioná una frase y usá `Destacar`/`H` o `Nota`/`N`; elegí entre tres colores
(acento, ámbar, verde); el panel `D` lista, edita, recolorea y borra. Se guardan
en `annotations.json` como offsets de caracteres + cita, así que sobreviven
cambios de tipografía y se reanclan por texto si el bloque cambió. `⌘F` abre
“Buscar en el libro”: índice efímero, búsqueda sin acentos/mayúsculas,
resultados con contexto, salto al match y `n`/`N` para recorrerlos.

M5 (segunda etapa) agrega el selector “Este libro / Todos”, que permite saltar a
una anotación de otro libro, y la exportación a Markdown desde el mismo panel.
También suma `G` / click en el porcentaje para abrir “Ir a…”, con salto por
porcentaje del libro o número de capítulo.

M6 (consultas) está cerrado: seleccioná una palabra o frase y usá **Consultar**
(o `K`) para Wiktionary, MyMemory o **OpenCode Go**. El panel tiene
Diccionario, Traducción, IA e Historial; las consultas online están apagadas por
defecto y la clave de IA vive solo en `OPENCODE_API_KEY`. El historial se guarda
en `consultas.json` (local, con límite y borrado); desde el detalle del historial
podés **Ir al texto** y volver a la frase. El centro **Configuración**
(`⌘,` o Ajustes) reúne lectura, biblioteca, anotaciones, consultas, privacidad y
avanzado.

## Capturas

| Vacía | Poblada |
|---|---|
| ![Biblioteca vacía](docs/img/library-empty.png) | ![Biblioteca poblada](docs/img/library-full.png) |
| **Búsqueda** | **Error de importación** |
| ![Búsqueda](docs/img/library-search.png) | ![Error](docs/img/library-error.png) |

| **Tipografía por libro** |
|---|
| ![Panel de tipografía con “Solo para este libro”](docs/img/reader-settings.png) |

| **Destacados de tres colores** | **Panel de notas** | **Nota abierta** |
|---|---|---|
| ![Tres colores de destacado](docs/img/reader-highlight.jpg) | ![Panel de destacados y notas](docs/img/reader-notes.jpg) | ![Editor de nota](docs/img/reader-note.jpg) |

| **Anotaciones globales** | **Ir a…** | **Búsqueda en el libro** |
|---|---|---|
| ![Panel global con anotaciones de dos libros](docs/img/reader-notes-global.jpg) | ![Ventana para ir a un porcentaje o capítulo](docs/img/reader-goto.jpg) | ![Resultados de búsqueda y coincidencia resaltada](docs/img/reader-search.jpg) |

| **Consulta: Wiktionary** | **Configuración · Consultas** | **Historial** |
|---|---|---|
| ![Diccionario integrado con definiciones](docs/img/reader-consulta.jpg) | ![Centro de configuración, sección Consultas](docs/img/reader-config.jpg) | ![Historial local de consultas](docs/img/reader-history.jpg) |

## Limitaciones conocidas

- CSS del libro ignorado por diseño (los temas mandan).
- Índice con anclas y enlaces internos: el salto intra/intercapítulo y los
  enlaces `<a href="#…">` funcionan; los enlaces externos todavía se ignoran.
- Sin paginado por columnas ni sinónimos/hyphenation.
- La búsqueda dentro del libro es substring sin regex/stemming y resalta solo la
  coincidencia activa; la selección de destacados no cruza párrafos (MVP).
- Tablas y `<dl>` se leen como bloques apilados; el SVG vectorial (paths) no se
  renderiza, solo el raster embebido en `<svg><image>`.
- La biblioteca no tiene watcher: se refresca a mano.
- El progreso por fracción y las preferencias por libro están desde M3; el
  filtro “Terminados” del roadmap original quedó fuera de alcance.
- La búsqueda sigue siendo substring sin regex/stemming; el índice de
  anotaciones global es una lista plana (sin búsqueda dentro de las notas).
- Exportación solo a Markdown; no hay PDF/DOCX ni sincronización entre
  dispositivos.
- Diccionario y traducción usan Wiktionary/MyMemory (online); la IA usa
  **OpenCode Go** y requiere `OPENCODE_API_KEY`. Online está apagado por defecto.
  El uso de la suscripción Go para lectura depende de sus términos y límites.
- La traducción se muestra solo en el panel; no reemplaza el texto del libro.
- El idioma de origen se deduce de `Book.lang`; el override manual queda para
  una fase posterior. La IA no admite preguntas libres, solo prompts
  predefinidos.

## Roadmap

- **M2 — Biblioteca (cerrada):** índice, portadas, grilla, búsqueda, orden,
  filtros, carpetas y migración de progreso.
- **M3 — Lectura fina (cerrada):** anclas, saltos, enlaces internos, progreso
  por fracción con ancla de bloque y ajustes por libro.
- **M4 — Extras (cerrado):** destacados/notas (spec 45) y búsqueda dentro del
  libro (spec 40).
- **M5 — Segunda etapa (cerrada):** anotaciones globales, tres colores, export
  Markdown (spec 50) y navegación por porcentaje/capítulo (spec 55).
- **M6 — Consultas y configuración (cerrado):** diccionario Wiktionary,
  traducción MyMemory, análisis con **OpenCode Go**, historial local (spec 60) y
  centro de configuración (spec 65). Online apagado por defecto; clave en
  `OPENCODE_API_KEY`.

## Modo diagnóstico

```bash
cargo run --release -- --inspect /ruta/libro.epub     # metadatos, spine, bloques, imágenes
cargo run --release -- --inspect tests/fixtures/edge.epub   # bordes del parser
cargo run --release -- --scan ~/Libros                # índice + portadas sin UI
cargo run --release -- --shot out.png --screen library
cargo run --release -- --shot out.png --screen library --search flores
cargo run --release -- --shot out.png --screen library --library-path /tmp/library.json
cargo run --release -- --shot out.png --chapter 13 libro.epub   # captura del lector
cargo run --release -- --shot out.png --anchor 6 libro.epub     # captura en un ancla
cargo run --release -- --shot out.png --panel settings libro.epub
```

## Privacidad

- Sin telemetría ni cuentas. La biblioteca, el progreso, las anotaciones y el
  historial de consultas viven en tu disco.
- Las consultas online están **apagadas por defecto** y solo salen con una acción
  explícita.
- La clave de IA se lee de `OPENCODE_API_KEY`; nunca se escribe en
  `config.json`, `consultas.json`, logs ni URLs.
- El historial guarda la consulta y la respuesta, no el párrafo de contexto.

## Licencia

Código bajo **MIT** ([`LICENSE`](LICENSE)). Literata © TypeTogether se
distribuye aparte bajo SIL Open Font License 1.1
(`assets/fonts/Literata-OFL.txt`).
