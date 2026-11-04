---
id: spec-30
title: Índice por ancla
status: draft
version: 1.0.0
updated: 2026-11-03
milestone: M3
depends_on: [spec-10, adr-0002]
---

# Spec 30 — Índice por ancla (M3)

## Problema

Hoy el índice mapea a archivos. Los libros con todo el texto en un XHTML (típico
de conversiones Calibre) tienen **una sola entrada**: el índice real está en
anclas (`index_split_000.html#sigil_toc_id_12`). Verificado con
`flores-del-mal.epub`: 2 capítulos, 1 entrada, 24.115 palabras.

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

### T1 — Capturar anclas en los dos parsers
- `Kind`/`Block` con `anchor`; tests: `<h2 id="x">` produce bloque con `anchor ==
  Some("x")`, tanto en XHTML válido como en HTML roto.
- `cargo test xhtml` verde; ningún test existente cambia de expectativa.

### T2 — Conservar fragmentos en el índice
- `TocEntry.anchor`; tests con el fixture (que ya tiene `cap3.xhtml#final`) y con
  un fixture nuevo que repita archivo con dos anclas distintas.
- `--inspect` debe imprimir el ancla cuando exista.

### T3 — Salto dentro del capítulo
- Mapa de anclas por frame + `pending_anchor`; test manual con `flores-del-mal`
  (una entrada por poema) y captura `--shot --anchor <id>`.
- Criterio: tras el salto, la línea del ancla queda en el tercio superior.

### T4 — Salto entre capítulos
- Resolver `(spine, anchor)`; si el capítulo cambia, cargar y saltar en el frame
  siguiente.
- Criterio: click en la entrada 40 de un índice de 101 lleva al párrafo exacto,
  no al inicio del archivo.

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
