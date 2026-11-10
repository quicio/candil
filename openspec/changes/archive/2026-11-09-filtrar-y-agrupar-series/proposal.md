## Why

La biblioteca ya lee series del OPF y puede ordenar por serie, pero no permite
aislar una serie concreta ni ver dónde empieza cada grupo. Con varias sagas en la
estantería, el orden por serie deja los libros uno detrás de otro sin una señal
visual clara.

## What Changes

- El filtro de la biblioteca suma las series detectadas y la opción **Sin serie**.
- Al elegir una serie, la grilla muestra solo esos libros y conserva búsqueda,
  orden y el resto de filtros disponibles.
- Con el orden **Serie**, la grilla muestra un encabezado por grupo con el nombre
  y la cantidad de libros; los libros sin serie quedan en un grupo final.
- Banderas `--order` y `--series` para capturas y verificación.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: filtro por serie/sin serie y encabezados de grupo en el
  orden por serie.

## Scope

**Incluido:** lista de series derivada de `library.json`, filtro por serie,
opción sin serie, encabezados de grupo, banderas de desarrollo y tests.

**No incluido:** detección de duplicados, virtualización de la grilla, búsqueda
full-text, edición de series desde la UI, agrupación por autor.

## Acceptance Criteria

- [x] El filtro lista cada serie una sola vez, ordenada y sin nombres vacíos.
- [x] Se puede filtrar por una serie y por **Sin serie**.
- [x] Con orden Serie se ve un encabezado por grupo con nombre y cantidad.
- [x] Los libros sin serie aparecen al final, bajo **Sin serie**.
- [x] Búsqueda y filtro de estado siguen funcionando juntos.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`, `src/library.rs`, `src/main.rs`.
- Sin cambios de datos ni dependencias; `library.json` no cambia.
- Docs: HANDOFF, ROADMAP, README y spec de biblioteca.
