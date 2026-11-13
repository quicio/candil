## Why

El usuario quiere pasar un EPUB a un Kobo, Kindle u otro lector sin salir de la
biblioteca. Kobo lee EPUB por USB; Kindle no, y Candil no debe fingir que sí.

## What Changes

- **Enviar a dispositivo…** en el menú contextual de cada tarjeta.
- Detecta un Kobo montado (carpeta `.kobo`) y ofrece **Enviar a Kobo**.
- Copia a `Books/` si existe, si no a la raíz; nunca sobrescribe.
- **Elegir carpeta o dispositivo…** para cualquier lector montado.
- Aviso explícito sobre Kindle: EPUB por USB no; Send to Kindle o conversión.
- La biblioteca y el EPUB original no se modifican.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: envío por copia a lectores externos.

## Scope

**Incluido:** diálogo, detección de Kobo, copia segura, nombres únicos, selector
genérico, aviso de Kindle, tests y ADR-0027.

**No incluido:** conversión AZW3/MOBI, Send to Kindle por email/API, borrado en el
dispositivo, sincronización o copia automática.

## Acceptance Criteria

- [x] El menú de tarjeta ofrece **Enviar a dispositivo…**.
- [x] Con un Kobo montado aparece **Enviar a Kobo** y copia el EPUB.
- [x] Nunca se sobrescribe un archivo existente en el dispositivo.
- [x] Se puede elegir cualquier carpeta/dispositivo manualmente.
- [x] El diálogo advierte que Kindle no lee EPUB por USB.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`, `src/main.rs`; sin dependencias nuevas.
- Docs: ADR-0027, HANDOFF, ROADMAP y README.
