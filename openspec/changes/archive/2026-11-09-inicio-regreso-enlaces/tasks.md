## 1. Inicio

- [x] 1.1 Agregar `StartupMode` a `Config` con default `LastBook` y test de compatibilidad. Verificación: `cargo test config`.
- [x] 1.2 Respetar el ajuste en `wrap_app`, con precedencia de path/`--resume`/`--screen library` y fallback si el libro falta. Verificación: manual con config vieja y nueva.
- [x] 1.3 Agregar el combo “Al iniciar” en Configuración → Biblioteca. Verificación: captura.

## 2. Volver al lector

- [x] 2.1 Agregar el botón ← en la barra del lector y cortar el render si cambia de pantalla. Verificación: captura a 980×780 y 720×600.
- [x] 2.2 Mantener `Esc` y el tooltip “Biblioteca (Esc)”. Verificación: revisión.

## 3. Enlaces externos

- [x] 3.1 Agregar `Book::external_url` con allowlist http/https/mailto/tel y tests. Verificación: `cargo test`.
- [x] 3.2 Propagar el clic externo desde `add_selectable_text` hasta el lector y mostrar el modal de confirmación. Verificación: captura con `--external-link-demo`.
- [x] 3.3 Abrir solo al confirmar, cerrar con Cancelar/Esc y rechazar esquemas inseguros. Verificación: revisión y tests.

## 4. Documentación y cierre

- [x] 4.1 ADR-0021 con el default, la precedencia y la confirmación de enlaces. Verificación: ADR en `docs/adr/`.
- [x] 4.2 Actualizar README, HANDOFF, ROADMAP, ARCHITECTURE y capturas. Verificación: `rg` sin referencias obsoletas.
- [x] 4.3 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y medir `dist`. Verificación: todo verde y < 8 MB.
- [x] 4.4 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate inicio-regreso-enlaces --strict` y `openspec archive`.
