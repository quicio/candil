---
id: spec-55
title: Navegación por porcentaje y capítulo
status: implementada
version: 1.0.0
updated: 2026-11-06
milestone: M5
depends_on: [spec-10, spec-35]
---

# Spec 55 — Navegación por porcentaje y capítulo

## Objetivo

Saltar a cualquier punto del libro sin recorrer capítulo por capítulo: un
porcentaje del libro o un número de capítulo.

## Alcance

1. Ventana **“Ir a…”** con:
   - slider de porcentaje (0–100) y botón “Ir al porcentaje”;
   - número de capítulo (1..N) y botón “Ir al capítulo”.
2. Se abre con `G` o haciendo click en el porcentaje de la barra superior.
3. `Esc` cierra la ventana.
4. El porcentaje se traduce a `(capítulo, fracción)` y se restaura con el mismo
   mecanismo de `Progress` (capítulo + ratio), sin ancla.
5. El número de capítulo usa numeración 1-based (como la barra), se clampea a
   `1..=N` y lleva al inicio del capítulo.

No entra:

- Saltar a un offset exacto escrito a mano.
- Miniaturas/preview del destino.
- Navegación por páginas (el lector es scroll continuo).

## Implementación

- Campos: `show_goto`, `goto_percent: f32`, `goto_chapter: usize`.
- `G` abre/cierra y cierra TOC/tipografía/búsqueda/notas para no apilar
  ventanas. Al abrir, `goto_percent = progress * 100` y
  `goto_chapter = chapter + 1`.
- Helper puro `chapter_for_percent(percent, chapters) -> (usize, f32)`:
  `pos = percent/100 * chapters`, capítulo = `floor(pos)` clampeado, fracción =
  `pos - capítulo`.
- `goto_percent` llama `load_chapter(chapter, Some(Progress { ratio: fracción,
  anchor: None, ... }))`; `goto_chapter` llama `load_chapter(chapter, None)`.
- La barra superior hace clickeable el porcentaje con tooltip “Ir a… (G)”.

## Tests

- `chapter_for_percent`: 0 %, 50 %, 100 % con N capítulos; capítulos chicos.
- El resto es UI; verificación visual con `--shot` y `--panel goto`.

## Criterio de salida

1. `G` abre la ventana y `Esc` la cierra.
2. 50 % lleva a la mitad del libro; 100 % al último capítulo.
3. “Ir al capítulo” acepta 1..N y no se sale de rango.
4. `cargo test` verde y captura revisada.
