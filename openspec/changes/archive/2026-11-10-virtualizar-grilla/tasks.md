## 1. Layout

- [x] 1.1 Agregar `GridItem`, alturas y `build_grid_items` con filas y encabezados. Verificación: test `grilla_arma_filas_y_grupos_por_serie`.
- [x] 1.2 Calcular el total con padding y mantener el render actual bajo el umbral. Verificación: captura de biblioteca normal.
- [x] 1.3 Virtualizar con `show_viewport` y `cards()` solo por fila visible. Verificación: capturas de 300 libros.

## 2. Cierre

- [x] 2.1 Verificar 300 libros en orden normal y Serie, con encabezados y acciones. Verificación: `--shot --library-path`.
- [x] 2.2 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 2.3 Actualizar HANDOFF, ROADMAP y README; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
