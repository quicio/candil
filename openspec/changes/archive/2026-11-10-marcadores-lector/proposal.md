## Why

El lector guarda el progreso automático, pero no permite volver a un pasaje
concreto sin buscarlo o recordar el porcentaje. Los destacados no cubren este
caso: un marcador es una posición sin rango de texto.

## What Changes

- `B` guarda un marcador en la posición actual; si ya existe uno a menos de 1 %
  en el mismo ancla, avisa y no duplica.
- `Mayús+B` abre el panel **Marcadores**, también accesible desde la barra del
  lector.
- El panel lista los marcadores del libro, permite saltar a cada uno y
  eliminarlos.
- Los marcadores viven en `BookEntry.bookmarks` dentro de `library.json`, usan el
  mismo `Progress` que la lectura y se conservan al escanear o reasociar.
- No se agrega un archivo nuevo: el respaldo de datos ya incluye `library.json`.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: marcadores de posición con atajo, panel, salto y borrado.

## Scope

**Incluido:** modelo en `library.json`, alta rápida, deduplicación, panel,
salto, borrado, preservación en escaneo/relink, atajos, ayuda, tests y ADR-0024.

**No incluido:** etiquetas editables, marcadores globales, exportación,
sincronización, marcadores automáticos por capítulo.

## Acceptance Criteria

- [x] `B` guarda un marcador en la posición actual y no duplica uno cercano.
- [x] `Mayús+B` abre el panel y la barra del lector también.
- [x] Saltar usa la posición exacta (capítulo, ancla y fracción).
- [x] Eliminar un marcador lo quita de `library.json`.
- [x] Un escaneo o reasociación conserva los marcadores.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/library.rs`, `src/app.rs`, `src/main.rs`.
- `library.json` gana `bookmarks` opcional; sin migración.
- Docs: ADR-0024, HANDOFF, ROADMAP, README y spec de lectura.
