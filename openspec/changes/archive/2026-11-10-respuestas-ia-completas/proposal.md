## Why

Con el default de 512 tokens, una explicación de IA se cortaba a mitad de frase
sin aviso. El panel mostraba una respuesta incompleta y el usuario no sabía si el
modelo había terminado o si Candil había truncado.

## What Changes

- El default de **máx. tokens** sube de 512 a **1536**.
- Un `config.json` con el default viejo (512) migra una vez a 1536; valores
  personalizados distintos de 512 se respetan.
- Si el proveedor corta por límite (`finish_reason=length`,
  `stop_reason=max_tokens` o `finishReason=MAX_TOKENS`), la respuesta agrega un
  **Aviso** con el valor actual y la acción para subirlo.
- Si la selección supera 2000 caracteres, el panel avisa que se enviarán los
  primeros 2000.
- La cita de la cabecera sigue siendo una vista previa; no implica que se envíe
  cortada.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: completitud de respuestas de IA y aviso de truncamiento.

## Scope

**Incluido:** default y migración de tokens, detección de corte por proveedor,
aviso en la respuesta, aviso de selección larga y tests.

**No incluido:** streaming, reintento automático con más tokens, chunking,
render Markdown de la respuesta.

## Acceptance Criteria

- [x] El default de tokens es 1536 y un valor viejo de 512 migra una vez.
- [x] Un valor personalizado distinto de 512 no se modifica.
- [x] OPENAI/Anthropic/Gemini detectan el corte por tokens.
- [x] La respuesta cortada muestra un aviso con el valor actual.
- [x] Una selección de más de 2000 caracteres avisa antes de enviar.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/config.rs`, `src/consulta.rs`, `src/app.rs`.
- Sin dependencias nuevas; `config.json` conserva `ai_max_tokens` y no guarda
  claves.
- Docs: HANDOFF, ROADMAP, README y spec del espacio de lectura.
