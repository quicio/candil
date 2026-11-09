## Why

Volver a un libro obliga a buscarlo en la grilla, y la biblioteca queda desactualizada si los EPUB cambian mientras Candil está en segundo plano. No hay un atajo para retomar la última lectura ni un refresco al volver a la ventana.

## What Changes

- Botón **Continuar leyendo** en la cabecera de la biblioteca, visible cuando hay un libro abierto recientemente y no faltante.
- Abre el libro con mayor `last_opened`; el tooltip muestra título y autor.
- La biblioteca se refresca incrementalmente al recuperar el foco, en un hilo, con debounce de 30 s.
- No se escanea si ya hay un escaneo en curso, si no hay carpetas o durante capturas.
- El refresco reutiliza el indicador “Escaneando…” y el escaneo incremental por mtime/size.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: se agregan el atajo de continuación y el refresco al recuperar foco.

## Scope

**Incluido:** botón Continuar leyendo, selección del último libro abierto, refresco al foco con debounce, feedback existente.

**No incluido:** watcher del sistema en tiempo real, series/colecciones, duplicados, virtualización de la grilla, búsqueda full-text de la biblioteca, “Terminados”.

## Acceptance Criteria

- [ ] Con un libro abierto antes, la cabecera muestra **Continuar leyendo** y abre ese libro.
- [ ] Sin libros abiertos, o si el último falta/está roto, el botón no aparece o usa el siguiente más reciente válido.
- [ ] El botón se compacta a **Continuar** en ventanas angostas y se oculta si no hay espacio.
- [ ] Al volver a la ventana en la pantalla de biblioteca, se dispara un escaneo incremental como máximo cada 30 s.
- [ ] No se dispara si ya hay un escaneo, si no hay carpetas o durante `--shot`.
- [ ] El escaneo no bloquea la UI y muestra “Escaneando…”.
- [ ] `cargo test`, `fmt`, `clippy -D warnings`, `scripts/check.sh` y la captura a 980×780 y 720×600 pasan.

## Impact

- `src/app.rs`: selección del reciente, botón de cabecera, estado de foco y refresco.
- `src/library.rs`: sin cambios; se reutiliza `rescan` incremental.
- `scripts/captures.sh` y `docs/img/library-*.png`: la biblioteca de capturas necesita un libro con `last_opened`.
- `docs/HANDOFF.md`, `docs/ROADMAP.md`, `README.md` y un ADR corto sobre el refresco al foco.
