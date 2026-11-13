## 1. Detección y copia

- [x] 1.1 Agregar `kobo_in`/`kobo_dir` y test con `.kobo`. Verificación: test `detecta_kobo_y_nombres_unicos`.
- [x] 1.2 Agregar `unique_destination` y test de sufijo ` (2)`. Verificación: mismo test.
- [x] 1.3 Agregar `copy_book_to_folder` con destino `Books/` o raíz y sin sobrescribir. Verificación: revisión.

## 2. UI

- [x] 2.1 Agregar **Enviar a dispositivo…** al menú de tarjeta y `CardAction::SendToDevice`. Verificación: captura `--send-device`.
- [x] 2.2 Diálogo con Kobo detectado, selector genérico, aviso de Kindle y Cancelar. Verificación: captura.
- [x] 2.3 Bandera `--send-device` para capturas. Verificación: `--shot`.
- [x] 2.4 Limpiar `device_send` al abrir un libro o volver a la biblioteca. Verificación: revisión.

## 3. Cierre

- [x] 3.1 Escribir ADR-0027 y actualizar HANDOFF, ROADMAP y README. Verificación: `openspec validate --specs --strict`.
- [x] 3.2 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
