---
id: adr-0008
status: aceptada
version: 1.1.0
updated: 2026-11-06
---

# ADR-0008 — Portadas: extraer, reducir, cachear en disco

## Contexto

Una estantería sin portadas es una lista. El EPUB declara la portada de tres
maneras distintas según cómo se generó:

1. EPUB3: `manifest item` con `properties="cover-image"`.
2. EPUB2/Calibre: `metadata meta name="cover" content="<id del item>"`.
3. Ni una ni otra: la imagen del primer XHTML del spine (a veces la cubierta está
   ahí, a veces es una ilustración).

Además las portadas suelen pesar 200 KB–1 MB y hay decenas: decodificarlas en
cada arranque es caro y llena memoria.

## Decisión

1. Resolución en cascada: `properties="cover-image"` → `meta name="cover"` →
   primer `<img>` (o `<svg><image>`) del primer documento del spine con imagen.
2. Reducción a PNG de **alto 320 px como máximo** (ancho proporcional, sin
   agrandar imágenes chicas) con la caja `image` y filtro Lanczos3.
3. Caché en disco: `covers/<fnv1a-de-ruta>.png`, escrito una sola vez al importar.
   Se usa FNV-1a y no `DefaultHasher` para que el nombre no cambie entre versiones
   de Rust.
4. Si no hay portada, la grilla dibuja un marcador tipográfico (iniciales del
   título sobre el color del panel) — nunca un hueco vacío.
5. El escaneo **nunca** decodifica una portada dos veces: si `size`/`mtime` del
   libro no cambiaron, se usa el caché y ni siquiera se abre el zip.

## Consecuencias

- Arranque de la biblioteca en milisegundos después del primer escaneo.
- +320 px por libro en disco (≈ 98 KB c/u): 100 libros ≈ 10 MB.
- El PNG del caché es un artefacto derivado: se puede borrar entero y se regenera.

## Alternativas descartadas

- **Guardar los bytes originales dentro de `library.json`**: hincha el índice y lo
  hace ilegible a mano; además base64 +30%.
- **Decodificar al renderizar**: simple pero inaceptable con decenas de portadas
  (200 ms+ por frame perdido).
- **Thumbnail en memoria solamente**: se pierde en cada arranque.

## Criterio de aceptación

Sobre el corpus de prueba, ≥ 90% de los libros con portada declarada muestran
portada, y el tamaño total de `covers/` es menor a 100 KB por libro.

## Implementación (2026-11-06)

`src/covers.rs` (miniatura + FNV + escritura) y `Book::cover_bytes()` (cascada +
validación + fallback SVG) están implementados y testeados. Sobre `~/Libros`:
41/43 portadas (95,3 %), 4,2 MB totales (~98 KB/libro), dentro del criterio.
