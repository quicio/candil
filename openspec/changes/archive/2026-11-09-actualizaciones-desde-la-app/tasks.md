## 1. Módulo de actualización

- [x] 1.1 Crear `src/update.rs` con `ReleaseInfo`, `Asset`, `current_version`, `compare_versions` y `parse_release`. Verificación: tests de versiones y parseo.
- [x] 1.2 Implementar la consulta a `api.github.com/repos/quicio/candil/releases/latest` con `ureq`, `User-Agent` y timeout. Verificación: `cargo check` y prueba manual del botón.
- [x] 1.3 Implementar `select_asset_for` y `platform_asset_name` para macOS arm64, Linux x86_64 y Windows x86_64; devolver `None` en otras plataformas. Verificación: tests con assets de ejemplo.
- [x] 1.4 Implementar `sha256_hex`, parseo de `SHA256SUMS.txt` y `download_release` con temporal + rename y permisos de ejecución. Verificación: test de checksum correcto/incorrecto.

## 2. Estado e hilos en la app

- [x] 2.1 Agregar `UpdateState`, `UpdateMsg`, `update_state` y `update_rx` a `Lector`; inicializar y llamar a `poll_update` en `update`. Verificación: `cargo check`.
- [x] 2.2 Implementar `check_for_updates` y `download_update` en hilos con `mpsc`, sin bloquear la UI. Verificación: la ventana sigue respondiendo durante la consulta.
- [x] 2.3 Agregar `open_external` y `reveal_path` multiplataforma. Verificación: abrir release y mostrar el archivo en macOS/Linux/Windows.

## 3. UI en Configuración → Avanzado

- [x] 3.1 Mostrar versión actual, texto de privacidad y botón “Buscar actualizaciones” con estado deshabilitado mientras hay una operación.
- [x] 3.2 Renderizar los estados: buscando, al día, disponible con fecha/novedades, descargando, descargado y error.
- [x] 3.3 Acciones “Descargar”, “Abrir release” y “Mostrar en Finder/Explorer”; aviso cuando no hay asset para la plataforma.
- [x] 3.4 Agregar `--update-demo` para inyectar una release simulada y verificar la UI sin red. Verificación: captura a 980×780 y 720×600.

## 4. Pruebas, documentación y cierre

- [x] 4.1 Tests de `compare_versions`, `parse_release`, `select_asset_for`, parseo de checksums y SHA-256. Verificación: `cargo test`.
- [x] 4.2 Escribir ADR-0016 con la API, la verificación, el destino en Descargas y las alternativas (auto-update, shasum externo, auto-check). Verificación: ADR en `docs/adr/`.
- [x] 4.3 Agregar `sha2` como dependencia directa y medir el `dist`. Verificación: sigue < 8 MB; `cargo tree -i sha2` no agrega un crate nuevo.
- [x] 4.4 Actualizar README, HANDOFF, ROADMAP, ARCHITECTURE y AGENTS; documentar que la instalación sigue siendo manual. Verificación: `rg` sin referencias obsoletas.
- [x] 4.5 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y `--inspect tests/fixtures/edge.epub`. Verificación: todo verde.
- [x] 4.6 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate actualizaciones-desde-la-app --strict` y `openspec archive`.
