## Why

Con una frase seleccionada, `K` abre el panel de consulta en Traducción, pero el usuario todavía tiene que pulsar “Traducir”. Para consultas rápidas falta un atajo que haga las dos cosas.

## What Changes

- **`⇧K`** (Mayúsculas + K) con una selección abre el panel en **Traducción** y lanza la consulta inmediatamente.
- Respeta `online_enabled`: si las consultas están apagadas, no sale nada y el panel muestra el aviso.
- `K` sigue igual: abre el panel y elige Diccionario para una palabra o Traducción para una frase.
- La ayuda y el tooltip de “Consultar” documentan el atajo.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: se agrega el atajo de traducción directa sobre la selección.

## Scope

**Incluido:** atajo ⇧K, apertura del panel en Traducción, ejecución inmediata, aviso de online apagado, documentación.

**No incluido:** traducir sin abrir el panel en una ventana aparte, cambiar el proveedor por defecto, atajos configurables, automatizar la consulta sin selección.

## Acceptance Criteria

- [ ] Con texto seleccionado, `⇧K` abre Traducción y lanza la consulta.
- [ ] `K` mantiene el comportamiento actual (Diccionario/Traducción según el texto).
- [ ] Con online apagado, `⇧K` no hace ninguna petición y muestra el aviso.
- [ ] Sin selección no pasa nada.
- [ ] La ayuda y el README listan `⇧K`.
- [ ] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`, `README.md`, `docs/HANDOFF.md`.
- Sin dependencias ni cambios de datos.
