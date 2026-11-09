## Why

Hoy la selección y los destacados viven dentro de un solo bloque: si una idea cruza dos o tres párrafos, el usuario solo puede subrayar una parte y crear anotaciones separadas. Para una lectura de estudio eso rompe la unidad del texto y ensucia el panel de notas.

Candil ya guarda offsets de caracteres y reancla por cita, así que extender ese contrato a un rango de bloques es el siguiente paso natural sin tocar el parser ni el render.

## What Changes

- Selección de texto que cruza párrafos: arrastrar más allá del borde del bloque extiende la selección al bloque siguiente, y **Mayúsculas + clic** en otro párrafo extiende el rango desde el ancla actual.
- El rango se resalta completo en todos los bloques que cubre, no solo en el primero.
- `Annotation` pasa a un rango multi-bloque con bloque/offset inicial y final, conservando la cita por bloque para reanclar.
- Migración transparente de `annotations.json` v1: las anotaciones existentes siguen funcionando como rangos de un solo bloque.
- El panel de destacados muestra una vista previa de una línea, indica cuántos párrafos cubre y salta al inicio del rango.
- Consultar y crear nota usan el texto completo seleccionado, con el contexto del primer bloque.
- Tests de migración, normalización de rangos, resolución y reanclaje con citas parciales.

## Capabilities

### New Capabilities

- Ninguna: la funcionalidad extiende el espacio de lectura existente.

### Modified Capabilities

- `reading-workspace`: la selección puede cruzar párrafos, el panel de anotaciones representa rangos multi-bloque y la persistencia conserva y migra esos rangos.

## Scope

**Incluido:** selección entre párrafos, resaltado completo del rango, persistencia v2 con migración, reanclaje por partes de cita, panel y salto al inicio, consulta/nota sobre el texto completo.

**No incluido:** selección entre capítulos, copiar todo el rango al portapapeles, editar el rango de una anotación existente (solo su nota y color), fusionar o dividir anotaciones, selección táctil, selección con teclado más allá del bloque actual, cambios al parser XHTML.

## Acceptance Criteria

- [x] Se puede seleccionar texto que empieza en un párrafo y termina en otro, arrastrando o con Mayúsculas + clic.
- [x] El rango completo se ve resaltado en todos los bloques cubiertos y el color elegido se aplica a todo el rango.
- [x] Al reabrir el libro, el rango se restaura con sus offsets; si el texto cambió, se reancla por las citas por bloque.
- [x] `annotations.json` v1 se migra sin perder anotaciones; las nuevas se guardan como v2.
- [x] Crear nota, editar nota, borrar y reubicar el EPUB siguen funcionando sobre rangos multi-bloque.
- [x] El panel muestra una vista previa de una línea y salta al inicio del rango sin fingir éxito si la cita no se encuentra.
- [x] Una selección de un solo bloque se comporta igual que hoy, sin regresiones.
- [x] La UI funciona a 980×780 y 720×600; la barra flotante no tapa el texto.
- [x] `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh` pasan; el `dist` sigue por debajo de 8 MB.
- [x] No se agregan dependencias.

## Impact

- `src/annotations.rs`: modelo v2, migración, rangos y reanclaje.
- `src/app.rs`: `Selection`, geometría de bloques, selección extendida, render, panel, salto y consulta.
- `src/store.rs`: sin cambios de formato; se conserva la escritura atómica.
- Tests nuevos en `src/annotations.rs` y `src/app.rs`; fixtures existentes (`mini.epub`, `edge.epub`).
- Documentación: `docs/HANDOFF.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md`, `README.md` y `docs/CONVENTIONS.md`.
- Captura nueva o actualizada en `docs/img/` con una selección de dos párrafos.
