---
id: adr-0008
status: propuesta
version: 1.0.0
updated: 2026-11-03
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
   primer `img` del primer documento con `<img>` del spine.
2. Reducción a PNG de **alto 320 px** (ancho proporcional) con la caja `image`.
3. Caché en disco: `covers/<hash-de-ruta>.png`, escrito una sola vez al importar.
4. Si no hay portada, la grilla dibuja un marcador tipográfico (iniciales del
   título sobre el color del panel) — nunca un hueco vacío.
5. El escaneo **nunca** decodifica una portada dos veces: si el PNG existe y el
   `mtime` del libro no cambió, se usa el caché.

## Consecuencias

- Arranque de la biblioteca en milisegundos después del primer escaneo.
- +320 px por libro en disco (≈ 60 KB c/u): 100 libros ≈ 6 MB.
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
