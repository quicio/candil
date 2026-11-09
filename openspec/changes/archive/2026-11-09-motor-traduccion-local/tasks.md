## 1. MyMemory

- [x] 1.1 Agregar `translation_model` a `ConsultaSettings` con default vacío y `clamp()`; tests de configuración. Verificación: `cargo test config`.
- [x] 1.2 Recortar a 500 caracteres con `truncate_at_word`; nunca partir palabras ni superar el límite. Verificación: tests de recorte con texto largo y sin espacios.
- [x] 1.3 Agregar aviso de traducción parcial y mapear el error de longitud a un mensaje amigable. Verificación: tests de `parse_mymemory` y respuesta con aviso.

## 2. Traducción local

- [x] 2.1 Agregar `AiPrompt::SoloTraducir` con system prompt estricto y routing de `run()` a Ollama. Verificación: test `traduccion_local_usa_ollama_sin_clave`.
- [x] 2.2 Reutilizar el proveedor Ollama (`localhost:11434/v1`, sin clave), modelo configurable y `ai_max_tokens` mínimo de 1024. Verificación: body de la petición en test.
- [x] 2.3 Permitir traducción/IA local con `online_enabled=false`; mantener bloqueo para proveedores remotos. Verificación: prueba manual con online apagado.
- [x] 2.4 Mensajes accionables para conexión fallida y 404 de modelo. Verificación: revisión de errores en la app.

## 3. UI y documentación

- [x] 3.1 Selector MyMemory/Ollama y campo de modelo en Configuración → Consultas, con aviso de límite o de privacidad. Verificación: capturas `--shot --panel config`.
- [x] 3.2 Mostrar motor/modelo y aviso de 500 caracteres en la pestaña Traducción; habilitar el botón según destino de red. Verificación: captura del panel.
- [x] 3.3 Actualizar README, HANDOFF, ROADMAP y ADR-0022. Verificación: `rg "Ollama|500"`.
- [x] 3.4 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.5 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate motor-traduccion-local --strict` y `openspec archive`.
