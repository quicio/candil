---
id: roadmap
status: vigente
version: 1.0.0
updated: 2026-11-03
---

# Roadmap

Cuatro milestones. Cada uno tiene criterio de salida **verificable**, no una
lista de deseos. Un milestone no se cierra si su criterio no se puede mostrar.

| # | Milestone | Estado | Specs | Criterio de salida |
|---|---|---|---|---|
| M1 | Lector | **Hecho** | [10](specs/10-lector-mvp.md) | 44 EPUB abren sin capítulos vacíos; imágenes resueltas; temas y progreso persistentes |
| M2 | Biblioteca | Siguiente | [20](specs/20-biblioteca.md), [21](specs/21-metadatos-y-portadas.md) | Escanear una carpeta de 50 EPUB en ≤ 5 s y verla como estantería con portadas, orden, filtro y apertura |
| M3 | Lectura fina | Después | [30](specs/30-indice-por-ancla.md), [35](specs/35-preferencias-por-libro.md) | Click en cualquier entrada del índice aterriza en el párrafo exacto, incluso si todo el libro es un solo XHTML |
| M4 | Extras | Futuro | [40](specs/40-busqueda.md), [45](specs/45-citas-y-marcadores.md) | Buscar una frase en el libro y saltar al resultado en < 300 ms |

## M1 — Lector (terminado)

Contrato vigente, detallado en `specs/10-lector-mvp.md`: parseo OPF+spine,
índice nav/ncx, modelo `Block`, render en columna centrada con scroll continuo,
5 temas, Literata embebida, progreso por libro.

Deuda conocida y aceptada: índice por archivo (no por ancla), sin CSS del libro,
sin paginado.

## M2 — Biblioteca (el trabajo siguiente)

Objetivo: dejar de abrir un archivo por vez. Una estantería que indexa carpetas
del disco, muestra portadas y permite abrir, ordenar, filtrar y buscar.

Entra:

- Índice persistente en `library.json` (ADR-0007): rutas vigiladas + entradas.
- Extracción de metadatos y portada al importar (ADR-0008).
- Grilla de portadas con título, autor y progreso.
- Orden (reciente, título, autor), filtro por idioma/estado, búsqueda incremental.
- Abrir desde la grilla y volver con `Esc`.
- Migración del progreso existente de `config.json` a `library.json`.

No entra (a M3/M4): índices por ancla, paginado, notas, sincronización, edición
de metadatos, estantes manuales.

Criterio de salida, medible:

1. Carpeta con 50 EPUB escaneada en ≤ 5 s (con caché de portadas).
2. ≥ 90% de los libros con portada visible (los que declaran una).
3. Reabrir la app no reescanear: biblioteca al instante desde `library.json`.
4. `cargo test` verde + capturas de estantería vacía, llena y filtrada.
5. `--inspect` sobre el corpus sin regresiones.

## M3 — Lectura fina

- Índice por ancla: entradas con `#fragmento`, salto al bloque con ese `id`.
- Preferencias por libro (tema, tamaño, interlineado) sin perder las globales.
- Salto a porcentaje y a capítulo por número.

## M4 — Extras

- Búsqueda dentro del libro con contexto y resaltado de resultados.
- Citas y marcadores exportables (texto + posición + título del libro).

## Orden de ataque recomendado

1. M2 tareas 1–3 (modelo + escaneo + portadas) sin UI: se prueban con `cargo test`.
2. M2 tareas 4–6 (pantalla, orden/filtro, apertura): primeras capturas.
3. M2 tareas 7–9 (migración, carpetas vigiladas, estados vacíos): cierre.
4. Recién entonces M3.

Regla: **una spec, un milestone, un PR lógico**. Si una tarea no encaja en la
spec, se discute antes de codear (flujo spec → código, nunca al revés).
