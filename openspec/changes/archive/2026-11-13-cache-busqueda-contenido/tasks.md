## 1. Caché

- [x] 1.1 Agregar `SearchCache` LRU con tope de 64 MB y `search_index_bytes`. Verificación: test `cache_de_indices_reutiliza_y_evicta`.
- [x] 1.2 Agregar `cache_key` con ruta + tamaño + mtime. Verificación: revisión.
- [x] 1.3 Compartir el caché con el worker y reutilizar `Arc<SearchIndex>`. Verificación: `cargo check` y búsqueda repetida.

## 2. Cierre

- [x] 2.1 Escribir ADR-0028 y actualizar HANDOFF, ROADMAP y README. Verificación: `openspec validate --specs --strict`.
- [x] 2.2 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
