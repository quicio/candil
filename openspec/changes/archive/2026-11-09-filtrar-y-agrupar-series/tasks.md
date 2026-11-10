## 1. Datos y filtro

- [x] 1.1 Agregar `Library::series_names` sin vacíos, duplicados ni distinción de mayúsculas. Verificación: test `series_names_ordenan_sin_vacios_ni_duplicados`.
- [x] 1.2 Extender `LibraryFilter` con `Series(String)` y `NoSeries` y un predicado reutilizable. Verificación: test `filtro_por_serie_y_sin_serie`.
- [x] 1.3 Listar series y **Sin serie** en el combo de filtro existente. Verificación: captura de biblioteca filtrada.

## 2. Encabezados

- [x] 2.1 Conservar el nombre crudo de la serie en `CardData`. Verificación: `cargo check`.
- [x] 2.2 Agrupar la grilla en orden Serie y dibujar encabezado con nombre y cantidad; **Sin serie** al final. Verificación: captura `--order series`.
- [x] 2.3 Agregar banderas `--order` y `--series` para capturas. Verificación: `--shot --screen library --order series --series …`.

## 3. Cierre

- [x] 3.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.2 Actualizar HANDOFF, ROADMAP, README y spec; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
