## Why

La grilla de biblioteca construía y dibujaba todas las tarjetas en cada frame.
Con 40 libros no se nota, pero una biblioteca de cientos acumula trabajo y
memoria innecesarios. La grilla debe seguir fluida y conservar los encabezados de
serie.

## What Changes

- A partir de **240 libros visibles**, la grilla usa `ScrollArea::show_viewport`
  y solo construye/dibuja las filas dentro del viewport (con una fila de margen).
- El layout se calcula antes como una lista de encabezados y filas; funciona
  igual con el orden **Serie** y sus encabezados.
- Por debajo del umbral se mantiene el render actual, con todas las tarjetas
  focusables.
- Tests de construcción de filas y grupos; capturas con 300 libros reales y con
  grupos de serie.
- No se agrega caché ni índice persistente.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: virtualización de la grilla para bibliotecas grandes.

## Scope

**Incluido:** umbral, layout de encabezados/filas, viewport, overscan, tests y
documentación del trade-off de foco.

**No incluido:** virtualización por columnas, scroll infinito, indexación de
contenido, cambios a `library.json` o al escaneo.

## Acceptance Criteria

- [x] Con menos de 240 libros se mantiene el render completo.
- [x] Con 240 o más solo se construyen las filas visibles (más una de margen).
- [x] El orden Serie sigue mostrando encabezados y la cantidad de libros.
- [x] La búsqueda, los filtros y las acciones de tarjeta siguen funcionando.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs` (layout y render de biblioteca).
- Sin cambios de datos, dependencias ni binario significativo.
- Docs: HANDOFF, ROADMAP y README.
