## Why

En un EPUB con notas al pie, tocar la referencia te saca del párrafo y te obliga a reconstruir dónde estabas. Los enlaces internos ya funcionan como saltos, pero no hay forma de leer la nota sin abandonar el texto ni de volver exactamente al punto anterior.

Una vista previa flotante resuelve las dos cosas: muestra el destino de la nota sin mover la lectura y deja un regreso explícito cuando el usuario decide ir al texto.

## What Changes

- Al hacer clic en un enlace interno con destino resoluble, se abre una **vista previa flotante** con el texto del bloque destino, el capítulo y dos acciones: **Ir al texto** y **Volver**.
- La vista previa funciona con notas en el mismo capítulo y en otro XHTML del spine; el capítulo destino se parsea al vuelo y no se muestra hasta confirmar.
- **Ir al texto** salta al ancla y guarda la posición anterior; aparece un botón **← Volver al texto** que restaura el capítulo, el bloque y la fracción de scroll.
- Escape, el botón Volver o desplazarse cierran la vista previa sin cambiar la posición.
- El comportamiento de los enlaces externos no cambia: siguen ignorados por ahora.
- Tests de extracción de texto de nota, detección de marcador y restauración de posición; verificación visual con el fixture de notas.

## Capabilities

### New Capabilities

- Ninguna: la funcionalidad extiende el espacio de lectura existente.

### Modified Capabilities

- `reading-workspace`: se agrega la vista previa de enlaces internos y el regreso al punto de lectura anterior.

## Scope

**Incluido:** clic en enlace interno, vista previa de una nota o bloque, salto confirmado, botón de regreso, cierre por Escape/scroll, soporte intra e intercapítulo.

**No incluido:** abrir enlaces externos, parsear `epub:type="noteref"` como metadato, editar notas al pie, historial de saltos de varios niveles, popovers anidados, notas flotantes dentro del texto.

## Acceptance Criteria

- [x] Un clic en una referencia a una nota muestra su texto sin mover el scroll ni cambiar de capítulo.
- [x] “Ir al texto” navega al destino y “← Volver al texto” restaura el capítulo, el bloque y la fracción anteriores.
- [x] “Volver”, Escape o desplazarse cierran la vista previa sin navegar.
- [x] Una nota en otro XHTML se previsualiza igual; el capítulo no cambia hasta confirmar.
- [x] Un destino sin texto muestra un aviso y no rompe la vista previa.
- [x] Los enlaces externos y los enlaces internos sin destino resoluble no abren la vista previa.
- [x] La UI funciona a 980×780 y 720×600; la vista previa no tapa la referencia ni el texto que se está leyendo.
- [x] `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh` pasan.
- [x] El `dist` sigue por debajo de 8 MB.

## Impact

- `src/xhtml.rs`: sin cambios de modelo; se reutilizan `Run.href`, `Block.anchor` y `Block.text()`.
- `src/epub.rs`: sin cambios; se reutilizan `link_target` y `chapter_blocks`.
- `src/app.rs`: estado de vista previa y posición anterior, render del popover, botón de regreso y atajos.
- `src/main.rs` y `src/app.rs`: flag de desarrollo para la captura.
- `tests/fixtures/notes.epub` y `scripts/make_fixture.py`: fixture sintético con nota al pie.
- `docs/adr/0015-vista-previa-enlaces.md`, `docs/**`, `README.md` y `scripts/captures.sh`.
