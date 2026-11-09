## Why

Candil solo podía usar OpenCode Go y la clave vivía únicamente en `OPENCODE_API_KEY`. El usuario quiere configurar la clave desde la app, que quede cifrada en reposo y poder elegir entre varios proveedores de IA sin editar variables de entorno ni reiniciar.

Guardar la clave en `config.json` en texto plano sería inaceptable. La solución usa el llavero nativo del sistema y mantiene `OPENCODE_API_KEY` como override para CI, scripts y pruebas.

## What Changes

- **Llavero del sistema:** campo en Configuración para guardar, reemplazar y borrar la API key. macOS usa Keychain; Linux, Secret Service; Windows, Credential Manager.
- **Override:** `OPENCODE_API_KEY` sigue teniendo prioridad sobre la clave guardada.
- **Proveedores:** OpenCode Go, OpenAI, Anthropic, Google Gemini, OpenRouter, Groq, Mistral, DeepSeek y Ollama local.
- **Protocolos:** OpenAI-compatible (`/chat/completions`), Anthropic (`/messages`) y Gemini (`/models/{model}:generateContent`).
- **Configuración:** selector de proveedor con base URL y modelo editables; al cambiar de proveedor se cargan sus defaults.
- **Seguridad:** la clave nunca se escribe en `config.json`, `consultas.json`, logs ni URLs; Gemini la envía en `x-goog-api-key`, no en la query.
- **Estado en la UI:** llavero, entorno o sin clave; nunca se muestra el valor.
- **Documentación y ADR:** ADR-0012, README, HANDOFF y ROADMAP.

## Capabilities

### New Capabilities

- `ai-providers`: catálogo de proveedores, defaults, selección y adaptadores de protocolo.
- `ai-credentials`: almacenamiento cifrado en el llavero, prioridad del entorno, estados y borrado.

### Modified Capabilities

- Ninguna: el comportamiento de diccionario/traducción no cambia.

## Scope

**Incluido:** keychain, selector de proveedor, protocolos OpenAI/Anthropic/Gemini, UI de clave, tests con transport mock, ADR y documentación.

**No incluido:** streaming de respuestas, preguntas libres, selección automática de proveedor por coste, OAuth, claves por libro, sincronización entre dispositivos, nuevos proveedores de diccionario/traducción.

## Acceptance Criteria

- [ ] La clave se guarda en el llavero y no aparece en `config.json`, `consultas.json`, logs ni URLs.
- [ ] `OPENCODE_API_KEY` gana sobre la clave guardada.
- [ ] El selector ofrece los nueve proveedores y actualiza base URL/modelo al cambiar.
- [ ] OpenAI-compatible, Anthropic y Gemini construyen request y parsean respuesta correctamente.
- [ ] Ollama funciona sin clave; el resto deshabilita la acción si falta.
- [ ] La UI muestra llavero/entorno/sin clave y permite borrar con confirmación.
- [ ] `cargo test`, `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings` pasan.
- [ ] El binario `dist` sigue por debajo de 8 MB.
- [ ] ADR-0012 documenta la decisión y el coste.

## Impact

- `Cargo.toml` / `Cargo.lock`: nueva dependencia `keyring` 4.2.
- `src/ai.rs`, `src/secrets.rs`, `src/config.rs`, `src/consulta.rs`, `src/app.rs`, `src/lib.rs`.
- `scripts/captures.sh`: `CANDIL_NO_KEYRING=1` para el HOME falso.
- `README.md`, `docs/HANDOFF.md`, `docs/ROADMAP.md`, `docs/CONVENTIONS.md`, `PRODUCT.md`, `docs/adr/0012-keyring-clave-ia.md`.
- Tests: catálogo de proveedores, bodies/parsers de los tres protocolos, Ollama sin clave y serialización sin campos de clave.
