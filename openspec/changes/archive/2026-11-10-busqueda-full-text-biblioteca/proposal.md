## Why

La búsqueda de la biblioteca solo mira título, autor y ruta. Para encontrar una
frase dentro de los libros hay que abrirlos uno por uno. La biblioteca necesita
una búsqueda en el contenido que no bloquee la UI ni llene el disco con un índice
permanente.

## What Changes

- En la biblioteca, **Enter** en el buscador inicia una búsqueda dentro del texto
  de los libros; el estado vacío también ofrece **Buscar en el contenido**.
- La búsqueda corre en un hilo, muestra progreso (`12 de 40 libros`) y se puede
  cancelar.
- Los resultados muestran libro, capítulo y un fragmento; al elegir uno, se abre
  el libro en ese capítulo y se resalta la coincidencia.
- Límites: 100 resultados totales, 20 por libro y 120 libros por búsqueda.
- No se crea ni persiste ningún índice: el texto se procesa en memoria y se
  descarta al terminar.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: búsqueda full-text bajo demanda en la biblioteca.

## Scope

**Incluido:** búsqueda en hilo, progreso, cancelación, resultados, salto al
pasaje, límites, bandera de capturas y tests del motor existente.

**No incluido:** índice persistente, búsqueda incremental entre sesiones,
stemming/regex, búsqueda en notas o sincronización.

## Acceptance Criteria

- [x] Enter con 3+ caracteres inicia la búsqueda en el contenido.
- [x] El estado vacío ofrece el mismo camino de forma visible.
- [x] La UI muestra progreso y permite cancelar sin bloquearse.
- [x] Los resultados indican libro, capítulo y fragmento.
- [x] Elegir un resultado abre el libro y resalta la coincidencia.
- [x] No se escribe ningún índice ni se modifica `library.json`.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`, `src/main.rs`; reutiliza `src/search.rs`.
- Sin dependencias ni datos nuevos.
- Docs: HANDOFF, ROADMAP y README.
