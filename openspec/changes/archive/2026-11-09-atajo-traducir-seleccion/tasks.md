## 1. Atajo

- [x] 1.1 Detectar `K` con Shift y selección; abrir consulta, forzar Traducción y lanzar la petición. Verificación: prueba manual.
- [x] 1.2 Mantener `K` sin Shift igual y respetar `online_enabled`. Verificación: prueba con online apagado.
- [x] 1.3 Documentar ⇧K en la ayuda, el tooltip de Consultar y el README. Verificación: `rg "⇧K"`.
- [x] 1.4 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 1.5 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate atajo-traducir-seleccion --strict` y `openspec archive`.
