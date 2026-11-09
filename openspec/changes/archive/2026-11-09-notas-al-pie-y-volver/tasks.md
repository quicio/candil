## 1. Estado y extracción

- [x] 1.1 Agregar `LinkPreview` y `back_spot: Option<Progress>` a `Lector`, y el flag de desarrollo `--link-preview`. Verificación: `cargo check`.
- [x] 1.2 Implementar `preview_text(blocks, anchor, max)` y `looks_like_note_marker(label)` como funciones puras. Verificación: tests de ancla en bloque, ancla vacía, texto vacío, truncado y marcadores.
- [x] 1.3 Extraer `current_progress()` de `persist()` con la lógica de `top_block` y `block_fraction`. Verificación: tests existentes de progreso verdes.

## 2. Clic y vista previa

- [x] 2.1 Cambiar el canal de clic de `link_target` a `LinkClick { spine, anchor, rect, label }` en `add_selectable_text` y `draw_block`. Verificación: `cargo check`.
- [x] 2.2 Implementar `open_link_preview`: resolver el texto en el capítulo actual o parsear el destino con `chapter_blocks`, y guardar título, texto y rect. Verificación: clic en una nota del fixture abre el popover sin cambiar de capítulo.
- [x] 2.3 Dibujar el popover con `egui::Area`: título, capítulo, texto con scroll, “Ir al texto”, “Volver” y “×”; colocarlo cerca del clic y recortarlo a la pantalla. Verificación: captura `--link-preview` a 980×780 y 720×600.
- [x] 2.4 Cerrar la vista previa con “Volver”, “×”, Escape o al desplazarse, sin tocar la posición. Verificación: revisión manual/captura.

## 3. Ir y volver

- [x] 3.1 Implementar `jump_from_preview`: capturar `current_progress()`, llamar a `jump_to` y guardar `back_spot`. Verificación: “Ir al texto” navega al ancla.
- [x] 3.2 Dibujar el botón flotante “← Volver al texto” cuando `back_spot` existe y restaurar el `Progress` con `load_chapter`/`pending_restore`. Verificación: volver al capítulo, bloque y fracción anteriores.
- [x] 3.3 Limpiar `back_spot` en navegación manual (índice, capítulos, búsqueda, anotaciones, ir a). Verificación: el botón desaparece tras navegar.
- [x] 3.4 Ajustar la ayuda de atajos y el tooltip si hace falta. Verificación: `Esc` cierra la vista previa primero.

## 4. Fixture, pruebas y documentación

- [x] 4.1 Agregar `tests/fixtures/notes.epub` con una nota intra-capítulo y una inter-capítulo desde `scripts/make_fixture.py`. Verificación: `Book::open` y `link_target` resuelven ambos destinos.
- [x] 4.2 Agregar tests de integración del fixture: resolución de anclas, extracción de la nota y capítulo destino. Verificación: `cargo test`.
- [x] 4.3 Escribir ADR-0015 con la vista previa genérica, el regreso de un nivel y las alternativas. Verificación: ADR en `docs/adr/`.
- [x] 4.4 Actualizar `README.md`, `docs/HANDOFF.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md` y `AGENTS.md`; quitar la limitación de notas al pie si aparece. Verificación: `rg` sin referencias obsoletas.
- [x] 4.5 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y `--inspect tests/fixtures/edge.epub`. Verificación: todo verde.
- [x] 4.6 Medir `dist`, confirmar < 8 MB y validar/archivar el cambio OpenSpec. Verificación: `openspec validate notas-al-pie-y-volver --strict` y `openspec archive`.
