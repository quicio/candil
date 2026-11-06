---
id: spec-30
title: Índice por ancla
status: aprobada
version: 1.1.0
updated: 2026-11-06
milestone: M3
depends_on: [spec-10, adr-0002, adr-0010]
---

# Spec 30 — Índice por ancla (M3)

> **Estado: T1–T4 implementadas (2026-11-06).** `Block.anchor` y
> `TocEntry.anchor` existen; el mapa por capítulo resuelve saltos intra e
> intercapítulo. `flores-del-mal` pasó de 1 a 8 entradas y `#6` aterriza en
> “Parte 2”. Falta T5 (enlaces internos clicables).

## Problema

Antes de T1/T2 el índice mapeaba a archivos. Los libros con todo el texto en un
XHTML (típico de conversiones Calibre) tenían **una sola entrada**: el índice real
está en anclas (`index_split_000.html#sigil_toc_id_12`). Verificado con
`flores-del-mal.epub`: 2 capítulos, 24.115 palabras. Hoy el parser y el índice ya
conservan las anclas (8 entradas para ese libro); falta el salto exacto.

Además los enlaces internos (`<a href="#nota3">`) hoy se ignoran: se pintan con
color de acento pero no hacen nada.

Además los enlaces internos (`<a href="#nota3">`) hoy se ignoran: se pintan con
color de acento pero no hacen nada.

## Objetivo

Que cualquier entrada del índice —y cualquier enlace interno— aterrice en el
bloque exacto, mismo dentro de un capítulo de 500 KB.

## Diseño

### 1. Anclas en el modelo

`Block` gana `anchor: Option<String>` (el `id` del elemento que lo originó).

- Parser estricto: al entrar a un elemento con `id`, se propaga a los bloques que
  genere (todos los del elemento, no solo el primero).
- Parser tolerante: se lee el atributo `id` de la etiqueta de apertura.

### 2. Índice con fragmento

`TocEntry` pasa a `{ title, spine, anchor: Option<String> }`. `resolve()` ya
descarta el `#`; ahora hay que **conservarlo** en el parseo de nav/ncx.

Dedupe actual (colapsar entradas consecutivas del mismo spine) debe cambiar: si
dos entradas comparten spine pero distinto ancla, son dos entradas válidas.

### 3. Salto en el render

El scroll es del lector (no hay `scroll_to` por widget porque la columna se
redibuja entera):

1. Al dibujar cada bloque con `anchor`, registrar `(anchor, y)` donde `y` es
   `ui.cursor().top()` menos el origen del contenido.
2. Al pedir un salto, guardar `pending_anchor: Option<String>` y hacer un frame
   extra (`ctx.request_repaint()`).
3. Después del `show` del `ScrollArea`, si hay `pending_anchor` y el mapa de
   anclas ya lo contiene, `scroll = y` y limpiar el pendiente.
4. Si el ancla era de otro capítulo: cargar capítulo, limpiar el mapa y reintentar
   el salto en el frame siguiente.

### 4. Enlaces internos

`<a href="#id">` dentro del capítulo → mismo mecanismo. Los `href` externos
(`http`, `mailto`) quedan fuera de alcance.

## Tareas

### T1 — Capturar anclas en los dos parsers (hecha)
- `Kind`/`Block` con `anchor`; tests: `<h2 id="x">` produce bloque con `anchor ==
  Some("x")`, tanto en XHTML válido como en HTML roto.
- `cargo test xhtml` verde; ningún test existente cambia de expectativa.

### T2 — Conservar fragmentos en el índice (hecha)
- `TocEntry.anchor`; tests con el fixture (que ya tiene `cap3.xhtml#final`) y con
  un fixture nuevo que repita archivo con dos anclas distintas.
- `--inspect` debe imprimir el ancla cuando exista.

Implementado: `parse_nav`/`parse_ncx` separan y percent-decodean el fragmento;
`build_toc` deduplica por `(spine, anchor)`; tests unitarios de fragmentos y
dedupe. Sobre `flores-del-mal`, `--inspect` pasó de 1 a 8 entradas (`#2`, `#6`,
`#101`, …).

### T3 — Salto dentro del capítulo (hecha)
- Mapa de anclas por frame + `pending_anchor`; test manual con `flores-del-mal`
  (una entrada por poema) y captura `--shot --anchor <id>`.
- Criterio: tras el salto, la línea del ancla queda en el tercio superior.

### T4 — Salto entre capítulos (hecha)
- Resolver `(spine, anchor)`; si el capítulo cambia, cargar y saltar en el frame
  siguiente.
- Criterio: click en la entrada 40 de un índice de 101 lleva al párrafo exacto,
  no al inicio del archivo.

Implementado: `chapter_anchors` por frame, `pending_anchor`/`current_anchor`,
`jump_to`, `--anchor <id>` y marcado de la entrada actual en el índice. El parser
estricto emite un bloque-ancla invisible para `<a id="…"></a>` vacíos (el caso de
`flores`); el render lo saltea. Verificado `#6` → “Parte 2” y `#final` desde otro
capítulo en `mini.epub`.

### T5 — Enlaces internos clicables
- Las `Run` con `link` guardan el `href`; click en la palabra salta.
- Fuera de alcance: enlaces externos, notas al pie flotantes.

## Criterio de salida

Sobre `flores-del-mal.epub` (1 archivo, N poemas): el índice lista los poemas y
cada entrada aterriza en su primer verso. Sobre `El Héroe de las Eras` (105
capítulos): el índice sigue funcionando y el salto es exacto al encabezado.

## Riesgos

- Muchos EPUB repiten el mismo `id` en capítulos distintos: el mapa es **por
  capítulo**, nunca global.
- Anclas en elementos vacíos (`<a id="x"></a>`): si el elemento no genera bloque,
  la ancla se pierde. Mitigación: heredar el ancla al bloque siguiente.
- Documentos gigantes: registrar 3.000 anclas por frame es barato (HashMap de
  `&str` → f32), pero medir con el peor caso antes de cerrar.
