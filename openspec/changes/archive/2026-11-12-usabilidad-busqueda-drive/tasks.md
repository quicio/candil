## 1. Búsqueda

- [x] 1.1 Agregar `library_content_enabled` y checkbox **Contenido** en la barra. Verificación: captura.
- [x] 1.2 Enter solo busca con el toggle activo; activarlo inicia la búsqueda. Verificación: revisión y captura.
- [x] 1.3 Panel de resultados con ancho/alto relativo a la ventana. Verificación: captura 1100×820.

## 2. Google Drive local

- [x] 2.1 Agregar `google_drive_candidates`/`google_drive_dir` con candidatos macOS y home. Verificación: test `candidatos_de_google_drive_incluyen_la_carpeta_del_home`.
- [x] 2.2 Agregar **Añadir Google Drive…** al menú Carpetas y al estado vacío. Verificación: revisión/captura.
- [x] 2.3 Indexar la carpeta con `start_scan` y explicar el fallback sin Drive. Verificación: revisión.
- [x] 2.4 Documentar ADR-0026 y que no hay OAuth ni credenciales.

## 3. Cierre

- [x] 3.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.2 Actualizar HANDOFF, ROADMAP y README; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
