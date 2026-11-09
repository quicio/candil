## 1. Metadatos

- [x] 1.1 Agregar `series`/`series_index` a `Book` y parsear Calibre y EPUB3 en `Book::open`. Verificación: tests con `mini.epub` y `notes.epub`.
- [x] 1.2 Agregar campos a `BookEntry`, `meta_version` y `META_VERSION`; copiar en `from_path_with_cover`. Verificación: test de `BookEntry` con mini.
- [x] 1.3 Hacer que el escaneo relea entradas con `meta_version` viejo conservando progreso, ajustes, portada y fechas. Verificación: revisión del camino `unchanged` y test de escaneo.

## 2. Biblioteca

- [x] 2.1 Agregar `series` a `CardData` y mostrar `Nombre · N` debajo del autor; ajustar la altura de la tarjeta. Verificación: captura con fixtures.
- [x] 2.2 Agregar `LibraryOrder::Series` y el orden por nombre/posición/título. Verificación: revisión manual con los fixtures.
- [x] 2.3 Verificar que las tarjetas sin serie no muestren una línea vacía y que el layout a 720×600 siga bien. Verificación: captura.

## 3. Fixtures, documentación y cierre

- [x] 3.1 Agregar serie Calibre a `mini.epub` y EPUB3 a `notes.epub` desde `scripts/make_fixture.py`. Verificación: `--inspect` y tests.
- [x] 3.2 Escribir ADR-0020 con variantes, `meta_version`, visualización y orden. Verificación: ADR en `docs/adr/`.
- [x] 3.3 Actualizar README, HANDOFF, ROADMAP, ARCHITECTURE y capturas. Verificación: `rg` sin referencias obsoletas.
- [x] 3.4 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y medir `dist`. Verificación: todo verde y < 8 MB.
- [x] 3.5 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate series-desde-opf --strict` y `openspec archive`.
