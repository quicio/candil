## 1. Worker

- [x] 1.1 Agregar estado de búsqueda, mensajes y cancelación con `AtomicBool`. Verificación: `cargo check`.
- [x] 1.2 Recorrer EPUB uno a uno con `Book::open` + `search::build`, con topes de 100/20/120. Verificación: revisión y búsqueda de prueba.
- [x] 1.3 Drenar el canal y pedir repaint mientras está ocupado. Verificación: captura con progreso/resultados.

## 2. UI y salto

- [x] 2.1 Enter en el buscador inicia la búsqueda; el estado vacío ofrece el botón. Verificación: captura.
- [x] 2.2 Panel con progreso, cancelar, resultados y aviso de límites. Verificación: captura `--library-content-query`.
- [x] 2.3 Abrir un resultado y resaltar el pasaje con las rutas existentes. Verificación: revisión de `open_library_content_hit`.
- [x] 2.4 Bandera `--library-content-query` para capturas. Verificación: `--shot`.

## 3. Cierre

- [x] 3.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.2 Actualizar HANDOFF, ROADMAP y README; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
