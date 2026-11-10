## 1. Datos

- [x] 1.1 Agregar `Bookmark` y `BookEntry.bookmarks` con `serde(default)`. Verificación: `cargo check`.
- [x] 1.2 Preservar marcadores en escaneo incremental y `relink`. Verificación: revisión de `library.rs`.
- [x] 1.3 Alta con id local, orden por capítulo/fracción y borrado. Verificación: test `marcadores_se_ordenan_por_posicion_y_se_borran`.

## 2. Lector

- [x] 2.1 `B` guarda la posición actual y evita duplicados cercanos. Verificación: test `marcador_compara_ancla_y_fraccion`.
- [x] 2.2 `Mayús+B` y el botón **Marcador** abren el panel; el panel lista, salta y elimina. Verificación: captura `--bookmarks`.
- [x] 2.3 Cerrar el panel al abrir otros paneles y con `Esc`. Verificación: revisión de handlers.
- [x] 2.4 Actualizar ayuda y README con `B`/`Mayús+B`. Verificación: `rg "Mayús+B"`.

## 3. Cierre

- [x] 3.1 Escribir ADR-0024 y actualizar HANDOFF, ROADMAP, README y spec. Verificación: `openspec validate --specs --strict`.
- [x] 3.2 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
