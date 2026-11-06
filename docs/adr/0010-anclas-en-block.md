---
id: adr-0010
status: aceptada
version: 1.0.0
updated: 2026-11-06
---

# ADR-0010 — Anclas en `Block` y en el índice

## Contexto

M1 mapea el índice a **archivos del spine**. Los EPUB convertidos por Calibre
suelen meter todo el libro en un XHTML y usar un índice real con anclas
(`index_split_000.html#sigil_toc_id_12`). En `flores-del-mal.epub` eso da 2
capítulos, 1 entrada de índice y 24.115 palabras: el lector no puede saltar a
ningún poema. Además los enlaces internos (`<a href="#nota3">`) se pintan como
enlaces pero no hacen nada.

El contrato `Block` no tiene noción de posición dentro del documento, y
`TocEntry` descarta el `#fragmento` al resolver la ruta.

## Decisión

1. `Block` gana `anchor: Option<String>` con el `id` del elemento que lo originó.
   Se propaga a **todos** los bloques del elemento (no solo al primero) y se
   hereda a los descendientes que no tengan `id` propio.
2. `TocEntry` gana `anchor: Option<String>`. `parse_nav`/`parse_ncx` conservan el
   fragmento y `Book::open` lo guarda. El dedupe deja de colapsar por spine: dos
   entradas con el mismo archivo y ancla distinta son válidas.
3. El salto se resolverá con un mapa **por capítulo** `anchor -> y` construido
   al dibujar. Si el ancla es de otro capítulo, se carga y se reintenta en el
   frame siguiente (`pending_anchor`). Un ancla en un elemento vacío se hereda al
   bloque siguiente.
4. Los `id` duplicados dentro de un capítulo no son un error: gana el primero.
5. `--inspect` imprime el ancla cuando existe.

## Consecuencias

Buenas:

- Los libros de un solo XHTML dejan de tener una única entrada útil.
- El salto es por contenido, no por píxeles: sobrevive cambios de fuente y
  ventana.
- Habilita enlaces internos y, más adelante, el anclaje de destacados y notas.

Malas:

- Cada bloque puede cargar un `String` de ancla; en capítulos con miles de
  bloques y un `id` en el contenedor, se repite el mismo ancla. Se acepta por
  simplicidad; se puede internar o compartir más adelante.
- Si el parser cambia y parte un bloque, el índice de bloque usado por M3 puede
  desplazarse; el ancla textual sigue siendo estable.
- El parser tolerante no captura anclas en elementos inline sueltos (caso raro);
  el estricto sí.

## Alternativas descartadas

- **Mapa de anclas separado del modelo**: evita tocar `Block`, pero obliga a
  mantener dos representaciones sincronizadas y no sirve para reanclar
  destacados por bloque.
- **Guardar solo el offset en píxeles**: se rompe al cambiar tipografía o ancho.
- **DOM completo**: contradice ADR-0001/0002 y el tamaño del proyecto.
- **Índice generado por heurística de encabezados**: no reemplaza el índice real
  del libro y pierde títulos y orden.

## Revisar si

- Los capítulos gigantes muestran un costo de memoria por `anchor` repetido.
- Se implementan enlaces externos o notas al pie flotantes.
