## Why

Desde el lector solo se vuelve con `Esc`; no hay un control visible y discreto. Al cerrar y reabrir, Candil siempre muestra la biblioteca aunque hubiera un libro abierto. Y los enlaces externos del EPUB se ignoran, así que una referencia a una web no hace nada.

## What Changes

- Icono **←** discreto en la barra del lector para volver a la biblioteca, con tooltip y el mismo efecto que `Esc`.
- Nueva opción **Al iniciar** en Configuración → Biblioteca:
  - **Último libro abierto** (por defecto): abre el último libro válido; si falta, la biblioteca.
  - **Biblioteca**: arranca siempre en la estantería.
- Los enlaces externos `http`, `https`, `mailto` y `tel` abren una **confirmación** y, al aceptar, se lanzan en la aplicación del sistema. `javascript:`, `data:` y `file:` se rechazan.
- Tests del allowlist de esquemas y del default de configuración.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: se agrega el regreso visible a la biblioteca y la confirmación de enlaces externos.
- `library-experience`: se agrega la elección de qué abrir al iniciar.

## Scope

**Incluido:** botón ←, ajuste de inicio, fallback a biblioteca, confirmación de enlaces externos, allowlist de esquemas, documentación.

**No incluido:** abrir enlaces sin confirmación, visor web interno, historial de enlaces, inicio con un libro específico distinto del último, atajos configurables.

## Acceptance Criteria

- [ ] El lector muestra un icono ← discreto que vuelve a la biblioteca; `Esc` sigue funcionando.
- [ ] El default abre el último libro válido; si no existe, abre la biblioteca.
- [ ] Elegir “Biblioteca” en Configuración → Biblioteca hace que el próximo arranque no abra un libro.
- [ ] `--resume` y `--screen library` siguen mandando para desarrollo/capturas.
- [ ] Un enlace `https` pide confirmación y abre el navegador solo al aceptar.
- [ ] `javascript:`, `data:` y `file:` no se abren.
- [ ] Cancelar o `Esc` cierra la confirmación sin abrir nada.
- [ ] `cargo test`, `fmt`, `clippy -D warnings`, `scripts/check.sh` y `dist < 8 MB` pasan.

## Impact

- `src/config.rs`, `src/app.rs`, `src/epub.rs`, `src/main.rs`.
- `docs/adr/0021-inicio-regreso-enlaces.md`, `README.md`, `docs/HANDOFF.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md`.
- `scripts/captures.sh` y dos capturas nuevas (inicio y enlace externo).
