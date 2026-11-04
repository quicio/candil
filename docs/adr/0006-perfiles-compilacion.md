---
id: adr-0006
status: aceptada
version: 1.0.0
updated: 2026-11-03
---

# ADR-0006 — Tres perfiles de compilación

## Contexto

Con el perfil release original (`lto = true` fat, `codegen-units = 1`,
`opt-level = "z"`) cualquier cambio de una línea tardaba **26 s**: LTO re-enlaza
los 249 crates en cada build. Con el disco de la máquina casi lleno, además,
aparecieron fallos de linker (`errno=28`).

Un lector no necesita LTO para andar: necesita iterar rápido y tener un artefacto
final chico cuando se distribuye.

## Decisión

| perfil | base | idea | cambio de 1 línea | binario |
|---|---|---|---|---|
| `fast` | dev | `opt-level = 2`, `debug = false`, incremental | **1.3 s** | 9.9 MB |
| `release` | — | `opt-level = 3`, `lto = "thin"`, `cgus = 16` | ~17 s | 7.7 MB |
| `dist` | release | `opt-level = "z"`, `lto = "fat"`, `cgus = 1` | ~56 s | 5.1 MB |

Regla de uso: `fast` para desarrollar, `dist` para el binario definitivo,
`release` solo si se quiere algo intermedio.

## Consecuencias

- Iteración 20× más rápida; el primer build de cada perfil es caro una sola vez.
- Ocupación de disco: cada perfil tiene su propio árbol. **No acumular los tres**:
  `cargo clean --profile dist` cuando no se esté distribuyendo.
- Los tiempos de referencia están medidos en esta máquina (MacBook Air aarch64);
  sirven para detectar regresiones, no como promesa.

## Alternativas descartadas

- **Solo `cargo run` (debug)**: 1 s por cambio pero el scroll y el render se
  sienten trabados; inaceptable para juzgar tipografía.
- **`sccache`**: útil, pero no resuelve el costo de LTO y agrega infraestructura.
- **Un solo perfil con `lto = false`**: compila rápido pero quedaba un binario de
  9 MB sin ganancia sobre `fast`.
