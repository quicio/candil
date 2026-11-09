## 1. Base y seguridad

- [x] 1.1 Agregar `keyring = "4.2"` a `Cargo.toml` y documentar la decisión, alternativas y coste en `docs/adr/0012-keyring-clave-ia.md`. Verificación: `cargo check` y ADR con medición de binario.
- [x] 1.2 Crear `src/secrets.rs` con `load/save/delete/status`, servicio `com.quicio.candil`, cuenta `ai-api-key`, prioridad de `OPENCODE_API_KEY` y `CANDIL_NO_KEYRING=1` para capturas/CI. Verificación: smoke test manual del llavero en macOS y tests sin tocar Keychain.

## 2. Catálogo y protocolos

- [x] 2.1 Crear `src/ai.rs` con `Provider`, `Protocol`, los nueve proveedores y `provider(id)` con fallback a OpenCode Go. Verificación: test `ids_unicos_y_protocolos`.
- [x] 2.2 Agregar `ai_provider` a `ConsultaSettings`, default `opencode`, clamp de ids inválidos y defaults de base URL/modelo por proveedor. Verificación: test `proveedor_invalido_cae_en_opencode`.
- [x] 2.3 Refactorizar `consulta.rs` para construir system/user una vez y armar bodies de OpenAI, Anthropic y Gemini. Verificación: tests de body de cada protocolo.
- [x] 2.4 Implementar parseo de `choices[0].message.content`, `content[].text` y `candidates[0].content.parts[].text`, con errores por proveedor. Verificación: tests `ia_usa_endpoint_y_parsea_respuesta`, `ia_anthropic_usa_messages_y_x_api_key`, `ia_gemini_usa_generate_content_y_header_de_clave`.
- [x] 2.5 Enviar la clave de Gemini en `x-goog-api-key` y no en la query; enviar `x-api-key`/`anthropic-version` en Anthropic y `x-opencode-session` solo en OpenCode. Verificación: tests de headers.
- [x] 2.6 Permitir Ollama sin clave y deshabilitar la acción en el resto si falta. Verificación: test `ia_ollama_no_exige_clave` y estados de UI.

## 3. Configuración y UI

- [x] 3.1 Agregar a Configuración → Consultas el selector de proveedor con defaults y campos editables de base URL/modelo. Verificación: captura `reader-config.jpg`.
- [x] 3.2 Agregar campo de API key (password), Guardar, Borrar con confirmación y estado llavero/entorno/sin configurar. Verificación: captura y keyring smoke.
- [x] 3.3 Usar la clave resuelta (entorno o llavero) en `run_consulta` y mostrar el proveedor activo/estado en el panel de Consulta. Verificación: `cargo test` y revisión visual.
- [x] 3.4 Mostrar el proveedor real en el historial y en `consulta_provider_label`; no guardar la clave en `consultas.json`. Verificación: tests de historial y serialización.

## 4. Documentación, capturas y cierre

- [x] 4.1 Actualizar README, HANDOFF, ROADMAP, CONVENTIONS, PRODUCT y AGENTS con llavero, proveedores, conteos y tamaño. Verificación: `rg` sin referencias obsoletas.
- [x] 4.2 Exportar `CANDIL_NO_KEYRING=1` en `scripts/captures.sh` y regenerar `docs/img/` con el HOME falso. Verificación: captura de Configuración sin error de llavero.
- [x] 4.3 Ejecutar `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 4.4 Medir `dist`, confirmar < 8 MB y actualizar la documentación. Verificación: 6.818.144 B.
- [x] 4.5 Validar y archivar este cambio OpenSpec. Verificación: `openspec validate proveedores-ia-clave --strict` y specs en `openspec/specs/`.
