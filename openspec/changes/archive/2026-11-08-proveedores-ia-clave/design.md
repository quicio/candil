## Context

Ver `proposal.md`. La IA actual está en `src/consulta.rs`: un único request OpenAI-compatible contra el `ai_base_url` configurado, etiquetado siempre “OpenCode Go”, y la clave solo se lee de `OPENCODE_API_KEY`. `ConsultaSettings` ya tiene base URL y modelo editables, así que la compatibilidad hacia atrás es sencilla.

La regla anterior prohibía la clave en `config.json`. Este cambio la mantiene, pero permite el llavero del sistema y conserva la variable de entorno como override.

## Goals / Non-Goals

**Goals:**
- Guardar/borrar la clave desde la UI en el llavero nativo.
- Mantener `OPENCODE_API_KEY` con prioridad.
- Soportar OpenAI-compatible, Anthropic y Gemini con adaptadores testeables.
- No romper configuraciones existentes ni el historial.
- Medir el coste en binario y dejarlo documentado en un ADR.

**Non-Goals:**
- Streaming, preguntas libres, OAuth, selección automática por coste.
- Claves por libro o por proveedor; una sola clave activa.
- Nuevos proveedores de diccionario/traducción.
- Cifrado propio o contraseña maestra.

## Decisions

### 1. `keyring` 4.2 para el llavero nativo

Se agrega la dependencia `keyring = "4.2"` (MIT OR Apache-2.0), que delega en Keychain, Secret Service y Credential Manager. Entrada: `service=com.quicio.candil`, `account=ai-api-key`. **Alternativa descartada:** archivo cifrado con contraseña maestra (fricción) o clave local (protección débil). Ver ADR-0012.

### 2. Registro de proveedores en `src/ai.rs`

`Provider { id, label, protocol, base_url, default_model, source_url, needs_key }` y `Protocol::{OpenAi, Anthropic, Gemini}`. La lista es estática y ordenada; `provider(id)` cae en OpenCode Go si el id no existe. La UI usa `ALL`; `config.clamp()` usa `provider()` para completar defaults.

### 3. Configuración aditiva y compatible

`ConsultaSettings` gana `ai_provider: String`. Configs viejas sin el campo quedan en `opencode`; `ai_base_url`/`ai_model` no vacíos se preservan. Al cambiar de proveedor en la UI se cargan sus defaults; después el usuario puede editarlos.

### 4. Tres adaptadores de protocolo

- **OpenAI-compatible:** `POST {base}/chat/completions`; `Authorization: Bearer`; OpenCode agrega `x-opencode-session`.
- **Anthropic:** `POST {base}/messages`; `x-api-key`, `anthropic-version: 2023-06-01`; body `system/messages/max_tokens`; parseo de `content[].text`.
- **Gemini:** `POST {base}/models/{model}:generateContent`; `x-goog-api-key`; `system_instruction/contents/generationConfig`; parseo de `candidates[0].content.parts[].text`.

El texto del pasaje y el system prompt se construyen una vez (`ai_prompt_texts`) y cada protocolo los empaqueta. **Alternativa descartada:** forzar todo por OpenAI-compatible, que no sirve para Anthropic/Gemini.

### 5. Clave fuera de archivos, logs y URLs

`src/secrets.rs` encapsula `load/save/delete/status`. `load()` prioriza `OPENCODE_API_KEY`, después el llavero. No se serializa ningún campo de clave. Los errores no incluyen el valor. Gemini usa header, no query. Los tests de config verifican que el JSON no contiene `api_key`/`ai_key`.

### 6. UI de configuración y estado

En Configuración → Consultas: selector de proveedor, campo `password`, Guardar, Borrar con confirmación y estado (“llavero”, “entorno”, “sin configurar”, error). El panel Consulta muestra el proveedor activo, el estado de la clave y deshabilita la acción si falta (salvo Ollama). `has_key` se calcula como `key.is_some() || !provider.needs_key`.

### 7. Capturas sin llavero

`CANDIL_NO_KEYRING=1` hace que `load()` devuelva “sin configurar” sin tocar el llavero. `scripts/captures.sh` lo exporta porque usa un `HOME` falso. En uso real no está definido y el llavero funciona.

### 8. Tests sin red ni llavero

Los tests usan `Mock` de `Transport` para OpenAI/Anthropic/Gemini y verifican URL, headers, body y parseo. No se toca el llavero real en `cargo test`. El smoke test del llavero se corrió a mano y se documenta en el ADR.

## Risks / Trade-offs

- **Keychain prompts** → esperado la primera vez; el estado y los errores son explícitos.
- **Linux sin Secret Service** → error visible; `OPENCODE_API_KEY` sigue disponible.
- **Modelos por defecto cambian** → el campo modelo es editable y el proveedor se guarda por id, no por índice.
- **Coste en binario** → +54 KB medidos; sigue bajo 8 MB y queda en ADR-0012.
- **Clave en memoria** → necesaria para el request; se limpia el input y no se persiste.
- **Proveedor con API distinta** → adaptadores y tests por protocolo; no se inventan headers comunes.

## Migration Plan

Sin migración de datos. `config.json` gana `ai_provider` con default `opencode`; el historial no cambia. Si el llavero no está disponible, se usa la variable de entorno. Rollback: quitar `ai_provider` y la dependencia; el resto del formato sigue siendo válido.

## Open Questions

Ninguna que cambie el alcance. Streaming, OAuth y selección automática quedan para cambios futuros.
