## 1. Detección

- [x] 1.1 Agregar `duplicate_key` con título y autor normalizados; título vacío no cuenta. Verificación: test `detecta_duplicados_por_titulo_y_autor`.
- [x] 1.2 Contar claves y filtrar solo grupos con más de una entrada. Verificación: mismo test.
- [x] 1.3 Sumar `LibraryFilter::Duplicates` al combo existente. Verificación: `--filter duplicates`.

## 2. UI

- [x] 2.1 Agregar `duplicate_count` a `CardData` e insignia **Posible duplicado ×N** en la portada. Verificación: captura de biblioteca.
- [x] 2.2 Tooltip con el criterio y recordatorio de que no se borra nada. Verificación: revisión visual.
- [x] 2.3 Mantener las acciones existentes (abrir, Finder, quitar) y no tocar el archivo. Verificación: revisión de `library_card`.

## 3. Cierre

- [x] 3.1 Agregar `--filter duplicates` para capturas. Verificación: `--shot --screen library --filter duplicates`.
- [x] 3.2 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.3 Actualizar HANDOFF, ROADMAP, README y spec; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
