## Context

Ver `proposal.md` y las specs. `store::data_dir()` contiene cuatro JSON de usuario y cachés (`covers/`, `backups/`). `store::write_atomic` ya escribe JSON de forma atómica. `zip` y `sha2` ya son dependencias directas (el segundo por el updater). `notes_query` no existe; el panel filtra hoy solo por libro/global.

## Goals / Non-Goals

**Goals:**

- Respaldo local portable y verificable de los datos de usuario.
- Importación segura: validar antes de tocar, backup previo, escritura atómica.
- Recargar el estado en caliente y volver a la biblioteca.
- Filtro de anotaciones por texto, tolerante a acentos/mayúsculas.

**Non-Goals:**

- Nube, cifrado del ZIP, backups programados, incluir EPUB/portadas/claves.
- Sincronización entre dispositivos o resolución de rutas de otra Mac.

## Decisions

### 1. ZIP + manifiesto con checksums

El respaldo incluye `config.json`, `library.json`, `annotations.json`, `consultas.json` y `candil-backup.json` con `format`, `app_version`, `created` y una entrada por archivo (`name`, `size`, `sha256`). El ZIP se escribe con `zip` y compresión deflate.

**Por qué no copiar la carpeta:** un directorio no es portable ni verificable. **Por qué no incluir `covers/` ni EPUB:** son caché o archivos externos; el índice apunta a rutas y las portadas se regeneran.

### 2. Lista blanca y validación previa

La importación solo acepta los cuatro nombres conocidos. Por cada archivo: tamaño máximo 64 MB, tamaño declarado, SHA-256 y parseo del JSON correspondiente (`Config`, `Library`, `Annotations`, `ConsultaHistory`). Cualquier fallo aborta antes de escribir.

**Por qué:** evita rutas arbitrarias (`../`), ZIP bombs y JSON corruptos. **Alternativa descartada:** confiar en el manifiesto; un respaldo manipulado podría escribir fuera de la carpeta.

### 3. Backup previo y escritura atómica

Si ya hay datos, se exporta el estado actual a `data_dir/backups/pre-import-<nanos>.zip` antes de escribir. Cada archivo se reemplaza con `store::write_atomic` (temporal + rename). Si una escritura falla, el backup previo es la vía de recuperación.

**Alternativa descartada:** escribir directo; un corte deja media restauración. **Alternativa descartada:** restaurar automáticamente desde el backup en el error; complica el manejo y puede ocultar el problema; se informa la ruta.

### 4. Recarga en caliente

Tras importar, `reload_user_data` vuelve a leer `Config`, `Library`, `Annotations` y `ConsultaHistory`; `go_library` cierra el libro/paneles y `start_scan` actualiza faltantes. `applied_theme = None` reaplica el tema importado. `dirty = false` evita un guardado inmediato innecesario; el `persist` de `go_library` reescribe el estado ya importado, no el anterior.

**Por qué no exigir reinicio:** la recarga es acotada y mejora la experiencia. **Alternativa descartada:** reiniciar la app; más simple pero deja al usuario sin confirmación visual.

### 5. Filtro de notas en memoria

`note_matches(query, quote, note, book, chapter)` normaliza con `collate` (minúsculas + sin acentos) y busca substring. Se aplica a `rows_book`/`rows_all` y a `export_entries`; el estado vive en `notes_query: String`, no se persiste. El campo aparece arriba de la lista, con “×” para limpiar y estado vacío específico.

**Alternativa descartada:** filtrar solo por cita; el usuario también busca por nota, libro o capítulo. **Alternativa descartada:** índice persistente; el panel es chico y se filtra por frame.

## Risks / Trade-offs

- **Rutas de otra Mac** → el backup restaura índices/rutas tal cual; los libros pueden quedar `missing` hasta reindexar. Se documenta.
- **Sin EPUB en el ZIP** → el respaldo es chico y portable, pero no es un backup de los libros. Se aclara en la UI.
- **Import parcial si falla una escritura** → backup previo obligatorio y escritura atómica por archivo; se informa la ruta.
- **Config importada con proveedor/ajustes distintos** → es intencional; el tema se reaplica y la clave sigue en el llavero local.
- **Filtro y export** → el Markdown refleja el filtro; el usuario ve el alcance en el campo.
- **ZIP manipulado** → lista blanca, tamaño máximo, checksum y parseo JSON.

## Migration Plan

Sin migración de datos. El formato del respaldo es v1. Rollback: importar el `pre-import-*.zip` generado antes de la restauración.

## Open Questions

Ninguna que cambie el alcance. Cifrado, nube y backups programados quedan fuera.
