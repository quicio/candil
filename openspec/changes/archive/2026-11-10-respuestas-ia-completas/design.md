## Context

`AI_MAX_TOKENS_DEFAULT` era 512. Los proveedores OpenAI-compatible devuelven
`finish_reason`, Anthropic `stop_reason` y Gemini `finishReason`, pero los
parsers solo extraían el texto. La UI no distinguía una respuesta completa de una
cortada.

## Goals / Non-Goals

**Goals:**

- Evitar el corte en el caso normal de una explicación de un pasaje.
- Avisar cuando el proveedor corta, con una acción concreta.
- No pisar valores personalizados que no sean el default viejo.

**Non-Goals:**

- Streaming ni reintentos automáticos.
- Cambiar el tamaño de la selección enviada (2000 caracteres).
- Renderizar Markdown en la respuesta.

## Decisions

### 1. Default 1536 y migración puntual

`AI_MAX_TOKENS_DEFAULT = 1536`. En `clamp`, un valor exactamente igual a
`AI_MAX_TOKENS_LEGACY` (512) se reemplaza por el nuevo default. Cualquier otro
valor se conserva. La migración corre en cada carga hasta que el usuario guarde,
pero es idempotente.

**Por qué 512 exacto:** fue el default de todas las instalaciones anteriores; un
valor distinto es una elección deliberada.

### 2. Detección por protocolo

`response_cut_by_tokens` relee el JSON de respuesta y busca:

- OpenAI-compatible y Ollama: `/choices/0/finish_reason == "length"`.
- Anthropic: `stop_reason == "max_tokens"`.
- Gemini: `/candidates/0/finishReason == "MAX_TOKENS"`.

Si el JSON es inválido no se marca; el parser ya habría fallado antes.

### 3. Aviso como sección

La respuesta agrega una sección **Aviso** al final, no modifica el texto del
modelo. Se guarda en el historial junto con la respuesta, así el detalle también
explica el corte.

### 4. Selección larga

`ai_prompt_texts` sigue recortando a `MAX_TEXT_CHARS` (2000). El panel muestra un
aviso solo cuando la selección real supera ese límite, para no confundir la vista
previa de 220 caracteres con el texto enviado.

## Risks / Trade-offs

- **Más tokens por consulta** → respuestas más largas y algo más de latencia.
  1536 es un punto medio para una explicación de un pasaje.
- **Migración de un 512 deliberado** → se acepta porque era el default viejo y
  producía cortes; el usuario puede volver a bajarlo y quedará guardado.
- **Proveedor sin campo de corte** → no se muestra aviso; no se inventa.

## Migration Plan

Sin cambios de esquema. La migración de 512 ocurre en `clamp` y se persiste en el
próximo guardado de `config.json`. Rollback: restaurar `AI_MAX_TOKENS_DEFAULT` y
quitar el aviso.
