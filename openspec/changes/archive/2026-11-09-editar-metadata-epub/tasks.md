## 1. Núcleo de edición

- [x] 1.1 Hacer `read_entry` visible al módulo de edición y crear `src/epub_edit.rs` con `read_info`, `save`, `CoverExt`, `CoverUpload` y `Edits`. Verificación: `cargo check`.
- [x] 1.2 Reescribir el ZIP conservando entradas, con `mimetype` primero y `Stored`, y sustituir OPF/portada. Verificación: tests `agrega_portada...` y `reemplaza_portada...`.
- [x] 1.3 Editar `dc:title`, `dc:creator` y `dc:language` con escape XML, y agregar/actualizar `cover-image` y `meta name="cover"`. Verificación: tests de OPF.
- [x] 1.4 Crear backup en `backup_dir`, validar con `Book::open` y portada, y reemplazar con `rename`/restauración en Windows. Verificación: tests de backup y apertura.

## 2. Portada e integración

- [x] 2.1 Preparar la imagen en la app: decodificar, limitar a 1600×2400 sin agrandar y codificar PNG/JPEG. Verificación: `prepare_cover` y preview.
- [x] 2.2 Agregar el modal `Editar EPUB` con título/autor/idioma, preview, elegir imagen, error y Guardar/Cancelar. Verificación: capturas `library-edit.png` a 980 y 720.
- [x] 2.3 Agregar `CardAction::Edit` al menú contextual y abrir el editor solo para libros válidos. Verificación: revisión visual.
- [x] 2.4 Actualizar `BookEntry` tras guardar preservando progreso, ajustes, fechas y anotaciones; refrescar portada y texturas. Verificación: código + tests de `epub_edit`.

## 3. Documentación, capturas y cierre

- [x] 3.1 Escribir ADR-0013 con backup, validación, alternativas y coste. Verificación: ADR en `docs/adr/`.
- [x] 3.2 Actualizar README, HANDOFF, ROADMAP, CONVENTIONS, PRODUCT y AGENTS con la edición, conteos y tamaño. Verificación: `rg` sin referencias obsoletas.
- [x] 3.3 Agregar la captura del editor a `scripts/captures.sh` y `docs/img/library-edit.png`. Verificación: captura revisada a 980 y 720.
- [x] 3.4 Ejecutar `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.5 Medir `dist`, confirmar < 8 MB y actualizar documentos. Verificación: 7.234.320 B.
- [x] 3.6 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate editar-metadata-epub --strict` y spec en `openspec/specs/epub-editing/`.
