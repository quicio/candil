## Why

Hoy Candil solo lee EPUB: si un libro no tiene portada o trae título/autor/idioma incorrectos, el usuario necesita Calibre u otra herramienta externa. La biblioteca indexa el archivo original y no ofrece forma de corregirlo.

Editar un EPUB es destructivo si se hace mal. Este cambio agrega una edición acotada con backup obligatorio, validación y reemplazo seguro.

## What Changes

- Acción **Editar metadatos y portada…** en el menú contextual de cada tarjeta.
- Edición de `dc:title`, `dc:creator` y `dc:language` del OPF.
- Agregar o reemplazar la portada con JPG/PNG/WebP, redimensionada a 1600×2400 como máximo.
- Backup en `data_dir/backups/` antes de tocar el EPUB.
- Reescritura del ZIP conservando todas las entradas, `mimetype` primero y sin compresión.
- Validación con `Book::open` y comprobación de la portada antes del reemplazo.
- Actualización de la entrada de biblioteca preservando progreso, ajustes, fechas y anotaciones.
- ADR-0013 y tests con fixtures con y sin portada.

## Capabilities

### New Capabilities

- `epub-editing`: lectura y edición segura de metadatos y portada dentro del EPUB, con backup y validación.

### Modified Capabilities

- Ninguna: el parseo, la biblioteca y las anotaciones mantienen su contrato.

## Scope

**Incluido:** título, autor, idioma, portada, backup, validación, reemplazo y actualización de biblioteca.

**No incluido:** borrar portada, editar serie/editorial/fecha/descripción, editar el XHTML, validación epubcheck, guardar una copia nueva, editar varios libros a la vez.

## Acceptance Criteria

- [ ] Se puede editar título, autor e idioma y el cambio se refleja al reabrir el EPUB.
- [ ] Se puede agregar una portada a un EPUB sin portada y reemplazarla en uno con portada.
- [ ] Antes de reemplazar se crea un backup y se valida que el EPUB nuevo abra y tenga portada.
- [ ] Si la edición falla, el original no se modifica.
- [ ] `mimetype` queda primero y sin compresión; las demás entradas se conservan.
- [ ] Progreso, ajustes y anotaciones siguen asociados al mismo libro.
- [ ] La UI muestra preview, error y confirmación; el editor funciona a 980×780 y 720×600.
- [ ] `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh` pasan.
- [ ] El `dist` sigue por debajo de 8 MB.

## Impact

- `src/epub_edit.rs`, `src/epub.rs` (`read_entry` visible al módulo), `src/lib.rs`, `src/app.rs`.
- `docs/adr/0013-editar-epub.md`, README, HANDOFF, ROADMAP, CONVENTIONS, PRODUCT, AGENTS.
- `scripts/captures.sh` y `docs/img/library-edit.png`.
- Tests nuevos con `mini.epub` y `edge.epub`.
