## Why

La búsqueda en el contenido dependía de presionar Enter sin una señal visible, y
el panel de resultados era demasiado angosto. Además, el usuario quiere leer EPUB
que ya tiene en Google Drive.

## What Changes

- La barra de biblioteca agrega un toggle **Contenido**; con él activo, Enter
  busca dentro del texto. El toggle también inicia la búsqueda al activarse.
- El panel de resultados usa un ancho relativo a la ventana (hasta 920 px) y más
  altura, en vez del ancho fijo pequeño.
- **Añadir Google Drive…** detecta la carpeta local de Google Drive para
  escritorio y la indexa como cualquier otra carpeta.
- Si no se detecta, explica cómo instalarlo o elegir la carpeta sincronizada.
- No hay OAuth ni API de Google; se documenta en ADR-0026.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: toggle de búsqueda en contenido, panel más amplio y
  carpeta local de Google Drive.

## Scope

**Incluido:** toggle, disparo por Enter/toggle, ancho/alto del panel, detección de
Google Drive local, mensaje de fallback, tests y documentación.

**No incluido:** OAuth, Google Drive API, descarga/caché propia, credenciales de
Google, sincronización bidireccional.

## Acceptance Criteria

- [x] La barra muestra un toggle **Contenido** visible.
- [x] Con el toggle activo, Enter inicia la búsqueda full-text.
- [x] Activarlo con una consulta válida inicia la búsqueda.
- [x] El panel de resultados es más ancho y alto.
- [x] **Añadir Google Drive…** indexa la carpeta local detectada.
- [x] Sin Google Drive local, el mensaje explica la alternativa.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`; sin dependencias nuevas.
- Docs: ADR-0026, HANDOFF, ROADMAP y README.
