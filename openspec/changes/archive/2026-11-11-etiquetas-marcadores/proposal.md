## Why

Los marcadores ya guardan un campo `label`, pero la UI siempre lo dejaba vacío y
mostraba el título del capítulo. Para posiciones de estudio o pasajes concretos,
una etiqueta corta (“Carta de Darcy”, “Primer giro”) hace la lista mucho más útil.

## What Changes

- El panel **Marcadores** agrega un campo **Etiqueta opcional** al crear.
- `B` sigue guardando sin etiqueta; el panel permite ponerle nombre.
- Cada marcador existente tiene **Editar** para renombrarlo.
- Una etiqueta vacía vuelve a mostrar el título del capítulo y el porcentaje.
- La etiqueta se guarda en `library.json` y se conserva con el escaneo/relink.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: etiquetas opcionales y editables en marcadores.

## Scope

**Incluido:** campo al crear, renombrar, edición vacía, persistencia, captura y
tests del modelo.

**No incluido:** etiquetas obligatorias, filtros por etiqueta, colores de
marcador ni marcadores globales.

## Acceptance Criteria

- [x] `B` sigue agregando un marcador sin etiqueta.
- [x] El panel permite crear con etiqueta y renombrar uno existente.
- [x] Una etiqueta vacía cae al título del capítulo.
- [x] La etiqueta persiste en `library.json`.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/library.rs`, `src/app.rs`.
- Sin cambios de esquema: `label` ya existía.
- Docs: ADR-0024 (actualización), HANDOFF, ROADMAP y README.
