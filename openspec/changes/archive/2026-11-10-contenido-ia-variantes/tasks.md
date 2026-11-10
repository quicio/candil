## 1. Parser

- [x] 1.1 Normalizar `content` string, array y objeto. Verificación: test `openai_parsea_texto_y_errores`.
- [x] 1.2 Aceptar `choices[0].text` legado. Verificación: mismo test.
- [x] 1.3 Error específico para `finish_reason=length`. Verificación: test `openai_explica_por_que_no_hay_texto`.
- [x] 1.4 Error específico para solo razonamiento y `finish_reason` en el genérico. Verificación: mismo test.

## 2. Cierre

- [x] 2.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 2.2 Actualizar HANDOFF, ROADMAP, README y spec; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
