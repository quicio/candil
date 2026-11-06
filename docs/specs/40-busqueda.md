---
id: spec-40
title: Búsqueda dentro del libro
status: implementada
version: 1.0.0
updated: 2026-11-06
milestone: M4
depends_on: [spec-10, spec-30, spec-35]
---

# Spec 40 — Búsqueda dentro del libro

## Objetivo

Encontrar una frase en el libro abierto y saltar al resultado. No es búsqueda
global de la biblioteca (eso ya existe sobre título/autor en M2).

## Decisiones (2026-11-06)

1. **Solo dentro del libro abierto.** La biblioteca conserva su búsqueda de
   metadatos; no se mezclan.
2. **Índice efímero.** Se construye la primera vez que se busca y vive en
   memoria. No se persiste en `library.json` (crecería con el texto de cada
   libro y obligaría a invalidarlo al cambiar el EPUB).
3. **Substring, sin regex ni stemming.** Normalización 1:1 por carácter:
   minúsculas + eliminación de acentos (`á→a`, `ñ→n`). Mantener la longitud
   permite reutilizar los offsets del texto normalizado para resaltar y saltar
   sin mapas auxiliares. No se colapsan espacios: una consulta con dos espacios
   solo coincide con dos espacios.
4. **Índice por bloques de texto.** Se guardan `original` y `normalizado` por
   bloque, con el índice real dentro de `Vec<Block>` (no el índice de la lista
   filtrada; si no, un `<img>` o `<hr>` correría todos los saltos).
5. **Tope de 200 resultados.** Una palabra común no debe congelar la lista; si
   se corta, la UI lo avisa y pide afinar la búsqueda.

## Implementación

- `src/search.rs`: `build(&mut Book)`, `normalize`, `search(&SearchIndex, query,
  max)`, `Hit { chapter, block, start, end, before, matched, after }`.
  Contexto de 60 caracteres a cada lado, con `…` cuando se recorta y saltos de
  línea reemplazados por espacios.
- **UI**: ventana “Buscar en el libro” con campo de texto y lista de resultados;
  `Buscar` en la barra o `⌘F`/`Ctrl+F` para abrir; `Esc` cierra. Cada resultado
  muestra el contexto con la coincidencia en color de acento y el capítulo.
- **Salto**: click/`Enter` carga el capítulo si hace falta (`pending_block_jump`)
  y refina el scroll a la altura aproximada del match dentro del bloque
  (`pending_search_jump`, fracción de caracteres). El match queda resaltado con
  el acento del tema (`search_highlight`).
- **n / N**: siguiente / anterior entre resultados, cuando el campo de búsqueda
  no tiene foco (después de saltar se suelta el foco para que funcionen).
- **Índice caliente**: buscar de nuevo no reparsea capítulos; recorre los textos
  normalizados en memoria.
- **Tests**: normalización 1:1, acentos/mayúsculas, varios bloques, tope y
  truncado, contextos, consulta corta y un test de integración sobre
  `mini.epub` que verifica que el bloque/offsets devueltos apuntan al texto real.
- **Captura**: `docs/img/reader-search.jpg` (resultados + coincidencia resaltada
  tras el salto).

## Presupuesto

- Búsqueda en libro de 500 KB: **< 300 ms con índice caliente**. Cumplido: la
  búsqueda es una pasada lineal sobre el índice; el coste grande es la
  construcción inicial (una sola vez por libro).
- Medición gruesa: abrir *Gödel, Escher, Bach* (14 MB), construir el índice,
  buscar y capturar 12 frames tardó **0.65 s** en `--shot`.
- Memoria: `original + normalizado` (≈2× el texto). Aceptable para un libro
  abierto; el índice se libera al volver a la biblioteca.

## Fuera de alcance

- Búsqueda entre todos los libros a la vez.
- Búsqueda difusa, semántica, regex o stemming.
- Índice persistente en disco.
- Resaltar *todas* las coincidencias en el texto: solo la activa.
