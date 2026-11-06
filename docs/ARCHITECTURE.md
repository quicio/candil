---
id: architecture
status: vigente
version: 1.6.0
updated: 2026-11-06
---

# Arquitectura

## Flujo de datos

```
archivo.epub
    │  zip
    ▼
container.xml ──► OPF (metadatos, manifest, spine) ──► nav.xhtml / toc.ncx
    │                                                     │
    │  spine[i] = ruta dentro del zip                     │ índice: (título, ruta#ancla)
    ▼                                                     ▼
XHTML crudo ──► sanitize() ──► ¿XML válido? ──sí──► parser estricto (roxmltree)
                                   │no
                                   └──► parser tolerante (scanner propio)
                                              │
                                              ▼
                                   Vec<Block> = { kind, runs[], src, alt }
                                              │
                                              ▼
                                   render egui: un LayoutJob por bloque
                                              │
                                              ▼
                                       columna centrada con scroll
```

Nada del EPUB llega a la UI sin pasar por `Vec<Block>`. Esa es la frontera: el
render nunca ve HTML, y el parser nunca ve píxeles.

## Responsabilidades

| módulo | hace | no hace |
|---|---|---|
| `epub.rs` | abre el zip, resuelve rutas, arma spine e índice con `#ancla`, entrega `Block`s y portada declarada | no decide estilos ni layout |
| `xhtml.rs` | convierte XHTML/HTML en `Block`s; dos parsers; entidades, whitespace, pre/tablas y SVG raster | no abre archivos ni conoce el zip |
| `library.rs` | índice de carpetas, escaneo incremental, metadatos, portadas y faltantes | no dibuja la UI |
| `annotations.rs` | destacados/notas por libro: offsets de texto, cita y nota; `annotations.json` atómico | no conoce píxeles ni la UI |
| `covers.rs` | miniatura sin upscaling, hash FNV y escritura en `covers/` | no conoce la UI |
| `store.rs` | JSON atómico, backup de corruptos y directorio de datos | no conoce el modelo |
| `app.rs` | ventana, biblioteca, lector, barra, scroll, paneles, teclado, caché de texturas | no parsea HTML |
| `theme.rs` | paletas (5 presets) y aplicación de `Visuals` | no conoce bloques |
| `config.rs` | settings globales en `config.json`; `Progress` con ratio/ancla/fracción y compatibilidad con el offset legado | no conoce el contenido |
| `fonts.rs` | registra Literata en 4 familias + monoespaciada | no decide tamaños |
| `lib.rs` | expone módulos para tests de integración | — |

## Modelo de bloques

```rust
pub enum Kind { Para, Heading(u8), Quote, Bullet, Numbered(u32), Rule, Pre, Caption, Image }
pub struct Run { text: String, bold: bool, italic: bool, small: bool, link: bool }
pub struct Block { kind: Kind, runs: Vec<Run>, src: Option<String>, alt: String, anchor: Option<String> }
```

Decisiones que lo sostienen:

- **Plano, sin árbol.** Un capítulo es una lista. Citas anidadas no se
  representan: se aplanan (suficiente para libros).
- **Estilo inline por run**, no por nodo: alcanza para negrita, itálica, versalitas
  y enlaces.
- **`src` solo para imágenes**, ya resuelto a ruta interna del zip por `epub.rs`.
- **`anchor`** guarda el `id` del elemento que originó el bloque (o el del
  contenedor heredado). `TocEntry.anchor` conserva el `#fragmento`; el salto
  exacto se resuelve con un mapa por capítulo (ADR-0010).
- **Enlaces internos:** `Run.link` + `Run.href`; el click se mapea con
  `Galley::cursor_from_pos` y `Book::link_target` resuelve `(spine, ancla)`.
- **SVG raster** (`<svg><image href|xlink:href>`): se mapea a `Kind::Image`; el
  SVG vectorial no se rasteriza (ADR-0009).
- **Tablas y `<dl>`**: no hay `Kind::Table`; cada celda/entrada es un `Para`
  propio para que el texto no se pegue.

### Cómo agregar un `Kind` nuevo

1. Agregar la variante en `xhtml::Kind`.
2. Mapearla en el parser estricto (`visit_node`) **y** en el tolerante
   (`parse_lenient`), o documentar por qué solo uno.
3. Renderizarla en `draw_block` (tamaño, color, espaciado antes/después).
4. Test unitario en `xhtml.rs` + test en `tests/epub_smoke.rs` si el fixture lo cubre.
5. Captura con `--shot` para validar el resultado a ojo.

## Render

Un `LayoutJob` por bloque, secciones por run, `wrap.max_width = ancho de columna`,
`justify = true` en párrafos y citas. `line_height = None` significa "alto de
fuente por defecto"; acá se usa `Some(size * line_height)` para control fino del
interlineado.

Columna centrada: `ScrollArea` vertical → `horizontal_top` con `add_space(pad)` →
`vertical` con `set_min_width/set_max_width(colw)`. El ancho se recalcula por
frame desde `settings.column_width` y el ancho disponible.

Imágenes: se decodifican con `image` y se suben a textura la primera vez que
aparecen; el caché (`HashMap<ruta, TextureHandle>`) se vacía al cambiar de
capítulo. Las que venían envueltas en `<svg><image>` llegan como `Kind::Image` y
siguen el mismo camino. Presupuesto: un capítulo a la vez.

## Anclas

Al dibujar un capítulo, cada `Block.anchor` registra su `y` en un mapa por
capítulo. Al elegir una entrada del índice se resuelve `(spine, anchor)`: si es
otro capítulo, se carga y se deja `pending_anchor` para el frame siguiente; si es
el actual, se ajusta el scroll. `current_anchor` marca la entrada activa. Un
`id` en un elemento vacío (`<a id="6"></a>`) se conserva como bloque-ancla
invisible para no perder el destino (ADR-0010).

## Anotaciones

Los destacados viven en `annotations.json`, separados de la biblioteca. Una
`Annotation` guarda `book` (ruta canónica), `chapter`, `block`, `start`/`end` en
**caracteres** dentro de `Block::text()`, `quote`, `note`, `color` y `created`.
El render resuelve cada anotación del capítulo con `resolve_annotation`: si los
offsets guardados siguen conteniendo la cita se usan tal cual; si no, se busca la
cita primero en ese bloque y después en todo el capítulo. Las que no aparecen no
se dibujan.

Cada bloque de texto se dibuja con un `TextEdit` multilínea inmutable (`&mut
&str`) con `.frame(false)`, margen cero y un `layouter` que reutiliza el
`LayoutJob` del lector. Eso habilita selección y copia sin perder tipografía ni
justificación. El rango de cursor se convierte a offsets del bloque descontando
el prefijo de viñeta/numeración; la barra flotante ofrece `Destacar` y `Nota`.
El resaltado se pinta con `TextFormat::background` en las secciones del job que
caen dentro de cada rango.

El panel “Destacados y notas” lista las del libro ordenadas por capítulo/bloque;
`Ir` carga el capítulo y resuelve el bloque por cita (`pending_block_jump`),
`Editar` abre la nota y `×` borra. `H`/`N`/`D` son los atajos; `Esc` sale de la
selección o cierra la ventana activa.

## Biblioteca

`app.rs` arranca en `Screen::Library` salvo que se pase un EPUB o `--resume`.
Como la grilla es responsiva (`ancho / 180`), con portada cacheada, título, autor y
progreso por fracción (capítulo + ratio + ancla). Las portadas se cargan como
texturas la primera vez y se cachean en `library_textures`; el escaneo y la lista
se hacen en el hilo de UI (43 libros ≈ 1.6 s la primera vez, aceptado en M2).
Click abre el libro; `Esc` en el lector vuelve y persiste progreso. La
búsqueda/orden/filtro se calculan en memoria sobre `Library::books`. Una fila de
chips muestra las carpetas vigiladas con `×` para dejar de vigilarlas (sin borrar
sus libros) y la barra muestra el contador de errores con tooltip.

## Estado de la app

`Lector` guarda: pantalla (`Library`/`Reader`), biblioteca, libro abierto
(`Book`), capítulo actual, bloques del capítulo, texturas del lector y de
portadas, `Config`, `Annotations`, selección/nota activas, offset de scroll,
altura del viewport, flags de panel y estado de guardado diferido.

Regla de oro: **el scroll es del lector, no del EPUB**. Se guarda por libro como
`{chapter, ratio, anchor, anchor_ratio}`: el ancla es el índice del bloque
visible superior y manda al restaurar; `anchor_ratio` interpola dentro de ese
bloque y `ratio` (`scroll / max_scroll`) sirve para la tarjeta y de fallback.
Así cambiar fuente, interlineado, ancho o tamaño de ventana no descoloca la
lectura, incluso en párrafos de varias pantallas. Las posiciones de bloque se
registran en coordenadas del contenido (`cursor + scroll − inner_rect.top()`),
no de pantalla.

## Persistencia

`~/Library/Application Support/lector/config.json` (macOS; `dirs::config_dir()`
en el resto). Se guarda con debounce de 700 ms cuando hay cambios, y en
`on_exit`. `src/store.rs` centraliza la escritura atómica: temporal → `sync_all` →
`rename`, de modo que un corte no deja el JSON a medias. Si el JSON está corrupto,
se renombra a `<archivo>.corrupt-<epoch>`, se arranca con valores por defecto y se
avisa en un toast: nunca se bloquea el arranque.

`library.json` vive en el mismo directorio y usa el mismo helper. Guarda
`schema_version`, carpetas vigiladas y una entrada por libro con ruta canónica,
metadatos, `size`/`mtime`, progreso, estado `missing` y `settings` opcional
(tema/tipografía solo para ese libro). Las portadas van a
`covers/<fnv1a>.png` (ADR-0007/0008). El progreso de los libros indexados se
escribe solo acá; `config.progress` se conserva para leer archivos viejos y se
vacía a medida que `migrate_progress_from` encuentra sus libros. El arranque
normal muestra la biblioteca; `--resume` reabre el último libro.

Precedencia de ajustes: `BookEntry.settings` → `Config.settings` → `Default`.
`+/-` y el panel Tipografía editan el override cuando existe; desmarcar "Solo
para este libro" lo borra y vuelve al global. El indicador `Aa·` de la barra
avisa que el libro tiene ajustes propios.

`annotations.json` vive en el mismo directorio y usa el mismo helper atómico.
Guarda `schema_version` e `items`; se escribe en cada alta/edición/borrado, no
en el debounce del progreso. Si está corrupto se renombra y se arranca vacío sin
bloquear el lector.

## Errores

Hoy: `Result<T, String>` con mensajes en español pensados para el usuario. Se
muestran en un toast, nunca en un `panic`. Si el manejo de errores crece, el paso
siguiente es `thiserror` con un enum `LectorError`; la decisión requiere su ADR
(no está tomada).

## Rendimiento

- Parseo por capítulo al abrirlo; no se precargan los 100+ capítulos.
- `--inspect` sirve para detectar capítulos que salen vacíos (síntoma de parser).
- `layout` de egui cachea galeras por contenido; crear `LayoutJob` por frame es
  barato mientras el texto no cambie.
- Tope práctico conocido: un libro con un solo XHTML de 500 KB (Calibre) tarda
  ~200 ms en aparecer; aceptable, no optimizar antes de M3.
