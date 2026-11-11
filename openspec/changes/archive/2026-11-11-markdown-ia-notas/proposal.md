## Why

Las respuestas de IA y las notas se mostraban como texto plano: los `**`, las
listas y las citas quedaban a la vista. Eso vuelve difícil leer una explicación
larga y choca con el cuidado tipográfico del lector.

## What Changes

- Nuevo renderer Markdown propio, sin dependencias, para egui.
- Soporta párrafos, encabezados, listas, citas, bloques de código, separadores,
  negrita, itálica y código inline.
- Se aplica a respuestas de IA en el panel y el historial, a la vista previa de
  notas y al editor de notas (vista previa completa).
- Diccionario y traducción siguen en texto plano.
- ADR-0025 y tests del parser.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: renderizado Markdown de respuestas de IA y notas.

## Scope

**Incluido:** parser de bloques, énfasis inline, renderer egui, integración en
consulta/historial/notas, tests y documentación.

**No incluido:** HTML, tablas, imágenes, enlaces clicables, listas anidadas
complejas ni un motor Markdown completo.

## Acceptance Criteria

- [x] Las respuestas de IA muestran encabezados, listas, citas y código con
      jerarquía visual.
- [x] `**negrita**`, `*itálica*` y `` `código` `` se renderizan sin marcas.
- [x] Las notas tienen vista previa inline y vista previa completa en el editor.
- [x] Diccionario y traducción conservan su texto plano.
- [x] No se agregan dependencias.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/markdown.rs`, `src/lib.rs`, `src/app.rs`.
- Sin cambios de datos ni dependencias.
- Docs: ADR-0025, HANDOFF, ROADMAP y README.
