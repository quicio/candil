## Why

Candil guardaba una sola clave y la reutilizaba al cambiar de proveedor de IA.
Eso podía enviar la clave de OpenAI a Anthropic o Gemini, y `OPENCODE_API_KEY`
actuaba como override para cualquier proveedor. Cada proveedor debe tener su
propia credencial.

## What Changes

- El llavero usa una cuenta por proveedor: `ai-api-key/<id>`.
- Cambiar de proveedor recarga su clave; el campo de entrada y la confirmación de
  borrado se limpian.
- `OPENCODE_API_KEY` solo aplica a OpenCode Go; se ignora para el resto.
- La clave histórica compartida se migra una sola vez a la cuenta del proveedor
  que la cargue primero.
- La UI aclara que cada proveedor guarda su propia clave.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `ai-credentials`: cuentas por proveedor, migración de la clave histórica y
  alcance limitado de la variable de entorno.

## Scope

**Incluido:** cuentas por proveedor, migración, recarga al cambiar proveedor,
alcance de `OPENCODE_API_KEY`, estado en la UI, ADR-0023 y tests del formato.

**No incluido:** validar la clave contra el proveedor, múltiples claves para el
mismo proveedor, claves por modelo, exportar/importar claves.

## Acceptance Criteria

- [x] Cada proveedor guarda y lee su propia cuenta del llavero.
- [x] Cambiar de proveedor no reutiliza la clave anterior.
- [x] `OPENCODE_API_KEY` solo se usa con OpenCode Go.
- [x] La clave histórica se migra una sola vez si la cuenta nueva no existe.
- [x] `config.json` y `consultas.json` siguen sin contener claves.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/secrets.rs`, `src/app.rs`.
- Sin dependencias ni campos nuevos en `config.json`.
- Docs: ADR-0023, HANDOFF, ROADMAP, README y spec de credenciales.
