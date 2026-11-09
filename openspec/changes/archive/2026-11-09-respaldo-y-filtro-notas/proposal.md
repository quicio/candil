## Why

La biblioteca, el progreso y las anotaciones viven solo en el disco de cada Mac. Si se pierde el disco o se cambia de equipo, no hay forma de llevarlos. Además, el panel global de anotaciones es una lista plana que se vuelve difícil de recorrer cuando hay muchas notas.

## What Changes

- **Exportar datos…** en Configuración → Avanzado: ZIP local con `config.json`, `library.json`, `annotations.json` y `consultas.json`, más un manifiesto con versión y checksums.
- **Importar datos…**: valida manifiesto, checksums, lista blanca y cada JSON; guarda un backup del estado actual y reemplaza los datos.
- Al importar, la app recarga biblioteca, ajustes, anotaciones e historial, vuelve a la biblioteca y lanza un escaneo incremental.
- **Filtro en Destacados y notas**: busca en cita, nota, libro y capítulo, sin acentos/mayúsculas.
- La exportación de Markdown respeta el filtro activo.
- Tests del ZIP, checksums, archivos no permitidos y del filtro de notas.

## Capabilities

### New Capabilities

- `data-backup`: exportación e importación local y validada de los datos de usuario.

### Modified Capabilities

- `reading-workspace`: el panel de anotaciones agrega un filtro por texto.

## Scope

**Incluido:** export/import ZIP, manifiesto, checksums, lista blanca, backup previo, recarga y rescaneo, filtro de notas y export filtrado.

**No incluido:** sincronización en la nube, incluir EPUB o portadas en el ZIP, exportar claves, cifrado del respaldo, backup automático programado, historial de versiones del respaldo.

## Acceptance Criteria

- [ ] Exportar crea un ZIP con los cuatro JSON y un manifiesto legible.
- [ ] Importar un respaldo válido restaura los datos, recarga la app y deja un backup previo.
- [ ] Un checksum incorrecto, un JSON inválido o un archivo fuera de la lista blanca aborta sin tocar los datos actuales.
- [ ] El ZIP no incluye EPUB, portadas ni claves.
- [ ] El panel de notas filtra por cita/nota/libro/capítulo, muestra un estado vacío claro y se puede limpiar.
- [ ] “Exportar…” de notas respeta el filtro.
- [ ] La UI funciona a 980×780 y 720×600.
- [ ] `cargo test`, `fmt`, `clippy -D warnings`, `scripts/check.sh` y `dist < 8 MB` pasan.

## Impact

- `src/backup.rs` nuevo, `src/lib.rs`, `src/app.rs`, `scripts/captures.sh`, `docs/img/`.
- `docs/adr/0019-respaldo-datos.md`, `README.md`, `docs/HANDOFF.md`, `docs/ROADMAP.md`, `docs/RELEASING.md` (si aplica).
- Tests nuevos en `src/backup.rs` y `src/app.rs`.
