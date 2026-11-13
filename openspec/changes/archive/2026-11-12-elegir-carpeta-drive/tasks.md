## 1. Picker

- [x] 1.1 Abrir `rfd::FileDialog` en `google_drive_dir()` y usar la carpeta elegida. Verificación: revisión de `add_google_drive_folder`.
- [x] 1.2 No indexar si el usuario cancela o si la carpeta ya existe. Verificación: revisión.
- [x] 1.3 Aviso sobre archivos no descargados y disponibilidad sin conexión. Verificación: revisión del toast.
- [x] 1.4 Tooltip en el menú y el estado vacío. Verificación: revisión.

## 2. Cierre

- [x] 2.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 2.2 Actualizar ADR-0026, HANDOFF, ROADMAP y README; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
