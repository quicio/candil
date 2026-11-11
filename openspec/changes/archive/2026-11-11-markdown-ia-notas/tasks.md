## 1. Renderer

- [x] 1.1 Agregar `src/markdown.rs` con parser de bloques e inline. Verificación: tests `parsea_bloques_basicos` e `inline_quita_marcas_y_aplica_estilos`.
- [x] 1.2 Renderizar encabezados, listas, citas, código, reglas y párrafos con `LayoutJob`. Verificación: `cargo test markdown`.
- [x] 1.3 Exponer `show` e `inline` y registrar el módulo en `lib.rs`. Verificación: `cargo check`.

## 2. Integración

- [x] 2.1 Aplicar `show` a respuestas de IA del panel y del historial. Verificación: captura del historial.
- [x] 2.2 Aplicar `inline` a la vista previa de notas y `show` al editor. Verificación: captura del editor de nota.
- [x] 2.3 Mantener diccionario/traducción en texto plano. Verificación: revisión de `ConsultaKind::Ia`.

## 3. Cierre

- [x] 3.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.2 Escribir ADR-0025 y actualizar HANDOFF, ROADMAP y README; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
