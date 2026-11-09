## 1. Módulo de respaldo

- [x] 1.1 Crear `src/backup.rs` con manifiesto, exportación ZIP y checksums. Verificación: test de export de los cuatro archivos.
- [x] 1.2 Implementar importación con lista blanca, tamaño, SHA-256 y parseo JSON antes de escribir. Verificación: tests de checksum y archivo no permitido.
- [x] 1.3 Crear backup previo en `backups/pre-import-*.zip` y escribir con `store::write_atomic`. Verificación: test con datos previos.

## 2. UI y recarga

- [x] 2.1 Agregar “Datos” en Configuración → Avanzado con Exportar/Importar y advertencia. Verificación: captura a 980×780 y 720×600.
- [x] 2.2 Implementar `export_data` e `import_data` con diálogos `rfd`. Verificación: export/import manual en un HOME de prueba.
- [x] 2.3 Recargar Config/Library/Annotations/ConsultaHistory, volver a la biblioteca, reaplicar tema y escanear. Verificación: la biblioteca refleja el respaldo.
- [x] 2.4 Mostrar un aviso modal con el resultado y la ruta del backup previo. Verificación: revisión visual.

## 3. Filtro de notas

- [x] 3.1 Agregar `notes_query` y el campo de filtro al panel, con “×” y estado vacío. Verificación: captura del panel.
- [x] 3.2 Implementar `note_matches` con `collate` y aplicarlo a las filas y a la exportación Markdown. Verificación: test del helper y export filtrado.
- [x] 3.3 Verificar que las acciones Ir/Editar/Borrar sigan funcionando sobre filas filtradas. Verificación: revisión manual.

## 4. Pruebas, documentación y cierre

- [x] 4.1 Agregar tests de export/import, backup previo, checksum, lista blanca y filtro. Verificación: `cargo test`.
- [x] 4.2 Escribir ADR-0019 con el formato, la validación, el backup previo y las alternativas. Verificación: ADR en `docs/adr/`.
- [x] 4.3 Actualizar README, HANDOFF, ROADMAP y capturas; agregar `--panel advanced` para la captura. Verificación: `rg` sin referencias obsoletas.
- [x] 4.4 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y medir `dist`. Verificación: todo verde y < 8 MB.
- [x] 4.5 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate respaldo-y-filtro-notas --strict` y `openspec archive`.
