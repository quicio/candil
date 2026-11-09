## Context

Ver `proposal.md`. `Book` abre el ZIP en modo lectura y `library.rs` indexa rutas canónicas. La edición debe mantener ese contrato y no agregar un motor EPUB general.

## Goals / Non-Goals

**Goals:** editar metadatos/portada, backup, validación y reemplazo seguro; actualizar biblioteca sin perder datos.

**Non-Goals:** borrar portada, editar XHTML/serie/editorial, epubcheck, copia nueva, edición masiva.

## Decisions

### 1. Módulo `epub_edit.rs`

`read_info` lee container.xml, OPF, metadatos y portada. `save` recibe `Edits { title, author, lang, cover }`, un `backup_dir`, y devuelve el backup. No conoce egui.

### 2. Reescritura completa del ZIP

Se crea un temporal en el mismo directorio, se escribe `mimetype` primero con `Stored`, se copian todas las entradas (omitiendo directorios y `mimetype`), y se sustituyen OPF/portada. Se conserva la compresión y permisos de cada entrada. **Alternativa descartada:** modificar una entrada in-place; el ZIP no lo permite de forma segura.

### 3. OPF por edición de texto acotada

Se reemplazan las etiquetas `dc:title`, `dc:creator` y `dc:language` preservando atributos; si faltan, se insertan antes de `</metadata>`. Para la portada se actualiza/inserta el `<item>` del manifiesto y el `<meta name="cover">`. Los valores se escapan como XML. **Alternativa descartada:** un writer XML nuevo (dependencia y pérdida de formato).

### 4. Portada

La app decodifica la imagen con `image`, la limita a 1600×2400 sin agrandar, y la codifica en PNG si la portada existente es PNG o en JPEG calidad 90 en el resto. Si el href actual no coincide con la extensión, se usa `cover.<ext>` y se actualiza el manifiesto. La preview se carga como textura egui.

### 5. Backup, validación y reemplazo

Backup en `data_dir/backups/<stem>-<nanos>.epub`. El temporal se valida con `Book::open` y `cover_bytes` si se agregó portada. Unix: `rename` atómico. Windows: `remove + rename` con restauración desde el backup si falla. Cualquier error borra el temporal y deja el original intacto.

### 6. Actualización de biblioteca

`BookEntry::from_path_with_cover` relee el EPUB; se copian `progress`, `settings`, `added` y `last_opened`. La ruta no cambia, así que las anotaciones siguen válidas. Se limpia el caché de texturas y se guarda `library.json`.

### 7. UI

Acción en el menú contextual de la tarjeta (solo libros no faltantes/rotos). Modal `panel_frame` con título/autor/idioma, preview, `Elegir imagen…`, advertencia de backup, error y Guardar/Cancelar. `--edit` abre el primero para capturas.

## Risks / Trade-offs

- **Backup ocupa disco** → es obligatorio; si falla, no se edita. El usuario puede borrar `backups/`.
- **OPF atípico** → los tests cubren OPF estándar con y sin portada; si el OPF no tiene metadata/manifest, se devuelve error y no se toca el original.
- **Portada SVG/GIF** → se convierte a JPEG y se actualiza el manifiesto; el archivo viejo puede quedar huérfano dentro del ZIP.
- **Windows no atómico** → backup + restauración si el rename falla.
- **Coste** → +416 KB; sigue bajo 8 MB.
- **Validación parcial** → no es epubcheck; el backup es la red de seguridad.

## Migration Plan

Sin migración. Los EPUB existentes se leen igual. Los backups se crean on-demand. Rollback: restaurar el backup manualmente.

## Open Questions

Ninguna que cambie el alcance.
