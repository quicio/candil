---
id: architecture
status: vigente
version: 1.0.0
updated: 2026-11-03
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
| `epub.rs` | abre el zip, resuelve rutas, arma spine e índice, entrega `Block`s por capítulo | no decide estilos ni layout |
| `xhtml.rs` | convierte XHTML/HTML en `Block`s; dos parsers (estricto y tolerante) | no abre archivos ni conoce el zip |
| `app.rs` | ventana, barra, scroll, paneles, teclado, caché de texturas | no parsea HTML |
| `theme.rs` | paletas (5 presets) y aplicación de `Visuals` | no conoce bloques |
| `config.rs` | settings + progreso en `config.json` | no conoce el contenido |
| `fonts.rs` | registra Literata en 4 familias + monoespaciada | no decide tamaños |
| `lib.rs` | expone módulos para tests de integración | — |

## Modelo de bloques

```rust
pub enum Kind { Para, Heading(u8), Quote, Bullet, Numbered(u32), Rule, Pre, Caption, Image }
pub struct Run { text: String, bold: bool, italic: bool, small: bool, link: bool }
pub struct Block { kind: Kind, runs: Vec<Run>, src: Option<String>, alt: String }
```

Decisiones que lo sostienen:

- **Plano, sin árbol.** Un capítulo es una lista. Citas anidadas no se
  representan: se aplanan (suficiente para libros).
- **Estilo inline por run**, no por nodo: alcanza para negrita, itálica, versalitas
  y enlaces.
- **`src` solo para imágenes**, ya resuelto a ruta interna del zip por `epub.rs`.

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
capítulo. Presupuesto: un capítulo a la vez.

## Estado de la app

`Lector` guarda: libro abierto (`Book`), capítulo actual, bloques del capítulo,
texturas, `Config`, offset de scroll, altura del viewport (para paginar con
teclado), flags de panel y estado de guardado diferido.

Regla de oro: **el scroll es del lector, no del EPUB**. Se guarda por libro como
`{chapter, offset}` en píxeles; si cambia el tamaño de fuente, el offset es
aproximado (limitación aceptada, ver M3).

## Persistencia

`~/Library/Application Support/lector/config.json` (macOS; `dirs::config_dir()`
en el resto). Se guarda con debounce de 700 ms cuando hay cambios, y en
`on_exit`. Si el JSON está corrupto, se arranca con valores por defecto: nunca
se bloquea el arranque por configuración.

M2 agrega `library.json` en el mismo directorio (ADR-0007) y las portadas en
`covers/<hash>.png`.

## Errores

Hoy: `Result<T, String>` con mensajes en español pensados para el usuario. Se
muestran en un toast, nunca en un `panic`. Si el manejo de errores crece, el paso
siguiente es `thiserror` con un enum `LectorError` (ver ADR pendiente en M2).

## Rendimiento

- Parseo por capítulo al abrirlo; no se precargan los 100+ capítulos.
- `--inspect` sirve para detectar capítulos que salen vacíos (síntoma de parser).
- `layout` de egui cachea galeras por contenido; crear `LayoutJob` por frame es
  barato mientras el texto no cambie.
- Tope práctico conocido: un libro con un solo XHTML de 500 KB (Calibre) tarda
  ~200 ms en aparecer; aceptable, no optimizar antes de M3.
