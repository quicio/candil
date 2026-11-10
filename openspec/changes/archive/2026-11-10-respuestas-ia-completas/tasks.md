## 1. Tokens

- [x] 1.1 Subir `AI_MAX_TOKENS_DEFAULT` a 1536. Verificación: test `max_tokens_viejo_se_actualiza_al_default_nuevo`.
- [x] 1.2 Migrar solo el valor viejo 512 sin pisar valores personalizados. Verificación: mismo test con 700.
- [x] 1.3 Ajustar el test de clamp existente. Verificación: `cargo test config`.

## 2. Aviso de corte

- [x] 2.1 Agregar `response_cut_by_tokens` para OpenAI/Anthropic/Gemini. Verificación: test `detecta_respuestas_cortadas_por_tokens`.
- [x] 2.2 Agregar sección **Aviso** con el límite actual. Verificación: test `ia_avisa_si_la_respuesta_se_corta`.
- [x] 2.3 Avisar selección de más de 2000 caracteres en el panel. Verificación: revisión visual.

## 3. Cierre

- [x] 3.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.2 Actualizar HANDOFF, ROADMAP, README y spec; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
