## Why

OpenCode Go empezó a devolver respuestas con `message.content` en formatos que el
parser no contemplaba (array de partes u objeto con `text`), y en algunos casos
`content` vacío con solo razonamiento interno. Candil mostraba “no devolvió texto”
sin explicar qué pasó ni qué hacer.

## What Changes

- `parse_openai` acepta `content` como string, array de partes, objeto con `text`
  y el formato legado `choices[0].text`.
- Si `content` está vacío y `finish_reason` es `length`, el error explica que se
  alcanzó el límite y sugiere subir **máx. tokens**.
- Si solo hay `reasoning_content`/`reasoning`/`thinking`, el error lo dice.
- El error genérico incluye `finish_reason` cuando existe.
- Tests con las variantes y los errores accionables.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: tolerancia del parser de IA a variantes de `content` y
  errores diagnósticos.

## Scope

**Incluido:** variantes de `content`, detección de razonamiento, `finish_reason`
en el error, tests.

**No incluido:** mostrar el razonamiento interno como respuesta, reintentos
automáticos, streaming ni cambios de proveedor.

## Acceptance Criteria

- [x] `content` string sigue funcionando.
- [x] `content` en array u objeto se une en texto.
- [x] `choices[0].text` legado se acepta.
- [x] Sin texto y con `finish_reason=length`, el error pide subir tokens.
- [x] Solo razonamiento interno produce un error específico.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/consulta.rs` y tests.
- Sin dependencias ni cambios de datos.
- Docs: HANDOFF, ROADMAP, README y spec de lectura.
