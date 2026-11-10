## 1. Llavero por proveedor

- [x] 1.1 Agregar `account_for(provider)` con `ai-api-key/<id>` y `entry_for`. Verificación: test de cuentas distintas.
- [x] 1.2 Cambiar `load`, `save` y `delete` para recibir el proveedor. Verificación: `cargo check` y llamadas de `app.rs`.
- [x] 1.3 Migrar la cuenta histórica `ai-api-key` una sola vez a la cuenta del proveedor. Verificación: revisión de `migrate_legacy`.

## 2. UI y entorno

- [x] 2.1 Cargar la clave al arrancar con el proveedor configurado. Verificación: `Lector::new`.
- [x] 2.2 Recargar clave, limpiar entrada y cancelar borrado al cambiar proveedor. Verificación: captura de Configuración.
- [x] 2.3 Limitar `OPENCODE_API_KEY` a OpenCode Go. Verificación: revisión de `env_key`.
- [x] 2.4 Aclarar en la UI que cada proveedor guarda su propia clave. Verificación: captura.

## 3. Cierre

- [x] 3.1 Ejecutar `cargo fmt`, `cargo test`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh`. Verificación: todo verde.
- [x] 3.2 Escribir ADR-0023 y actualizar HANDOFF, ROADMAP, README y spec; validar y archivar el cambio OpenSpec. Verificación: `openspec validate --specs --strict`.
