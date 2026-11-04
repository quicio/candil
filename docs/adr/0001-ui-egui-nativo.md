---
id: adr-0001
status: aceptada
version: 1.0.0
updated: 2026-11-03
---

# ADR-0001 — UI nativa con egui, no webview

## Contexto

El EPUB es un zip con XHTML + CSS. La opción "obvia" para renderizarlo fiel es un
motor de navegador embebido (`wry`/WKWebView, o Tauri). La exigencia del proyecto
es distinta: **liviano, minimalista, con control total de la tipografía**, y un
único binario sin framework de sistema.

Requisitos: abrir en < 300 ms, binario pequeño, temas propios que manden sobre el
CSS del libro, y una sola columna de lectura con scroll continuo.

## Decisión

UI nativa con `egui`/`eframe` y un traductor propio XHTML → `Vec<Block>`. El
lector decide toda la apariencia; del EPUB solo se toma estructura y texto.
Sin webview, sin CSS, sin JavaScript.

## Consecuencias

Buenas:

- Binario único de 5.1 MB, arranque instantáneo, cero dependencias de sistema.
- Control total: interlineado, ancho de columna, justificado y temas son
  parámetros del lector, no una pelea con el CSS del libro.
- Un modelo de datos chico (`Block`) que se testea sin GUI.

Malas:

- Fidelidad imperfecta en EPUBs con maquetación rara (tablas complejas, notas al
  pie flotantes, poesía con sangrías).
- Hay que mantener dos parsers de HTML (ver ADR-0002).
- Sin paginado nativo: se resuelve con scroll continuo.

## Alternativas descartadas

- **wry / WKWebView**: máxima fidelidad y temas por CSS inyectado, pero el
  render depende del motor del SO y el control tipográfico se limita a CSS dentro
  de un webview. Contradice "minimalista y liviano".
- **Slint**: declarativo y prolijo, pero el texto largo y el HTML del EPUB se
  convertían igual a mano, con menos control de layout que egui.
- **TUI (ratatui)**: la más liviana, pero incompatible con "fuentes, colores de
  fondo y de letra" como presets del lector.
