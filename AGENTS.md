# AGENTS.md

Punto de entrada para cualquier agente que tome este repo. Leer en este orden:

1. **`docs/HANDOFF.md`** — estado real, reglas de trabajo, cómo verificar.
2. **`docs/ROADMAP.md`** — qué sigue y cuándo se considera terminado.
3. **`docs/ARCHITECTURE.md`** — módulos, modelo de bloques, puntos de extensión.
4. **`docs/CONVENTIONS.md`** — estilo, tests, commits.
5. La spec del milestone activo en `docs/specs/`.
6. Los ADR que toquen el área: `docs/adr/`.

## Comandos

```bash
cargo test                                  # 14 unit + 6 integración (fixture propio)
cargo run --profile fast -- <libro.epub>    # iterar: ~1.3 s por cambio
cargo build --profile dist                  # artefacto final: 5.1 MB
cargo run --profile fast -- --inspect <libro.epub>          # diagnóstico de parseo
cargo run --profile fast -- --shot out.png --chapter 13 <libro.epub>   # captura visual
```

## Reglas duras

- El disco de la máquina está al límite: **no dejes perfiles de compilación
  acumulados**. `cargo clean --profile dist` / `--profile release` antes de builds
  grandes. Un link que falla con `errno=28` es disco lleno, no código.
- No subir `egui`/`eframe` sin ADR: 0.37 es una reescritura (`App::ui`, sin
  `TopBottomPanel`). Ver `docs/adr/0003`.
- No agregar motor CSS ni webview: el modelo `Block` es el contrato.
- Toda dependencia nueva necesita justificación en un ADR (peso, licencia, mantenimiento).
- Nada de `unwrap()` en rutas de datos de usuario (EPUB roto, archivo faltante).
- Cada tarea termina con: `cargo test` verde, `cargo fmt`, y verificación visual
  con `--shot` si tocó UI.
