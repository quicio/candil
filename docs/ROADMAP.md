---
id: roadmap
status: vigente
version: 1.2.0
updated: 2026-11-06
---

# Roadmap

Cuatro milestones. Cada uno tiene criterio de salida **verificable**, no una
lista de deseos. Un milestone no se cierra si su criterio no se puede mostrar.

| # | Milestone | Estado | Specs | Criterio de salida |
|---|---|---|---|---|
| M1 | Lector | **Hecho** | [10](specs/10-lector-mvp.md) | 43 EPUB abren sin capítulos vacíos; 2279/2279 imágenes; temas y progreso persistentes |
| M1.1 | Endurecimiento | **Hecho** | [15](specs/15-endurecimiento-lector.md) | Espacios, entidades, pre/tablas, SVG raster y config atómica verificados sobre el corpus |
| M2 | Biblioteca | **Casi cerrada** | [20](specs/20-biblioteca.md), [21](specs/21-metadatos-y-portadas.md) | Escanear una carpeta de 50 EPUB en ≤ 5 s y verla como estantería con portadas, orden, filtro y apertura |
| M3 | Lectura fina | Después | [30](specs/30-indice-por-ancla.md), [35](specs/35-preferencias-por-libro.md) | Click en cualquier entrada del índice aterriza en el párrafo exacto, incluso si todo el libro es un solo XHTML |
| M4 | Extras | Futuro | [40](specs/40-busqueda.md), [45](specs/45-citas-y-marcadores.md) | Buscar una frase en el libro y saltar al resultado en < 300 ms |

## M1 — Lector (terminado)

Contrato vigente, detallado en `specs/10-lector-mvp.md`: parseo OPF+spine,
índice nav/ncx, modelo `Block`, render en columna centrada con scroll continuo,
5 temas, Literata embebida, progreso por libro.

Deuda conocida y aceptada: índice por archivo (no por ancla), sin CSS del libro,
sin paginado.

## M1.1 — Endurecimiento (terminado)

Spec [15](specs/15-endurecimiento-lector.md). Cerró los bordes que la auditoría
del 2026-11-05 encontró en libros reales: espacios en límites inline, entidades
Latin-1/griegas/matemáticas, `<pre>` con indentación, tablas/`<dl>` sin pegar
celdas, `<svg><image>` raster y escritura atómica de `config.json`.

Verificación: 43 EPUB, 0 fallos, **0 capítulos vacíos**, 2279/2279 imágenes;
25 unit + 7 integración al cerrar M1.1 (hoy 41 + 8 con M2); clippy sin
warnings. Capturas de `flores-del-mal`
(título) y `Elantris`/`El camino de los reyes` (mapas SVG) revisadas.

## M2 — Biblioteca (casi cerrada)

Implementado y medido el 2026-11-06 sobre `~/Libros` (43 EPUB en la medición; hoy
quedaron 40 tras limpiar duplicados):

- Modelo `library.json` con `schema_version`, rutas canónicas, `size`/`mtime`,
  `missing` y progreso; escritura atómica compartida en `src/store.rs`.
- Escaneo recursivo incremental: **1.57 s** la primera vez, **0.02 s** con caché.
- Portadas cacheadas: **41/43 (95,3 %)**, **4,2 MB** en `covers/`.
- Pantalla con grilla, apertura, `Esc` para volver, búsqueda incremental, 4
  órdenes y 4 filtros; estados vacío/sin resultados; marcador cuando no hay
  portada.
- `--scan <carpeta>` para probar el núcleo sin UI; `--resume` para abrir el
  último libro (el arranque normal es la biblioteca).

- Migración de progreso a `library.json` (T8): `config.json` ya no recibe nuevas
  posiciones y las viejas se vuelcan al arrancar/después de escanear.
- Gestión de carpetas y errores (T9): chips con `×` para dejar de vigilar,
  contador de errores con tooltip y estados vacíos diferenciados.

Pendiente para cerrar: T10 (`--screen library`/`--library-path` y captura
filtrada) y T11 (medir 50 libros y cerrar métricas).

Objetivo original: dejar de abrir un archivo por vez. Una estantería que indexa
carpetas del disco, muestra portadas y permite abrir, ordenar, filtrar y buscar.

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

0. M1.1 cerrado (spec 15): no construir la biblioteca sobre texto pegado o SVG
   vacío.
1. M2 tareas 1–3 (modelo + escaneo + portadas) sin UI: se prueban con `cargo test`.
2. M2 tareas 4–6 (pantalla, orden/filtro, apertura): primeras capturas.
3. M2 tareas 7–9 (migración, carpetas vigiladas, estados vacíos): cierre.
4. Recién entonces M3.

Regla: **una spec, un milestone, un PR lógico**. Si una tarea no encaja en la
spec, se discute antes de codear (flujo spec → código, nunca al revés).
