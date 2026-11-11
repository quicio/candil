## 1. Datos

- [x] 1.1 Agregar `BookEntry::set_bookmark_label` con recorte y test. Verificación: test `marcadores_se_ordenan_por_posicion_y_se_borran`.
- [x] 1.2 Mantener la etiqueta al escanear y reasociar. Verificación: revisión de `library.rs`.

## 2. Panel

- [x] 2.1 Campo **Etiqueta opcional** y botón de alta. Verificación: captura del panel.
- [x] 2.2 Acción **Editar** y modo renombrar con Guardar/Cancelar. Verificación: captura/revisión.
- [x] 2.3 Fallback al capítulo cuando la etiqueta está vacía. Verificación: captura.
- [x] 2.4 `B` sigue guardando sin etiqueta. Verificación: revisión de `add_bookmark_here`.

## 3. Cierre

- [x] 3.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.2 Actualizar ADR-0024, HANDOFF, ROADMAP y README; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
