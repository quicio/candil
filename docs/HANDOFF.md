---
id: handoff
status: vigente
version: 1.3.0
updated: 2026-11-06
---

# Handoff — estado del proyecto y cómo seguir

## Qué es

Lector de EPUB de escritorio en Rust: una columna, scroll continuo, tipografía
propia, temas prearmados. egui/eframe para la ventana, parser XHTML propio para
el contenido. Sin webview, sin motor CSS.

**M1 (lector) y M1.1 (endurecimiento, spec 15) están terminados y verificados.**
**M1, M1.1 y M2 están cerrados y verificados. M3 está en curso:** T1/T2 de
anclas están hechas (`Block.anchor`, fragmentos del índice); faltan los saltos
exactos y el progreso por fracción.

## Estado medido (no estimado)

Medición del 2026-11-06 sobre `~/Libros` (43 EPUB; el corpus de `~/Downloads` es
el mismo):

| métrica | valor | cómo se midió |
|---|---|---|
| corpus real | 43 EPUB, 0 errores de parseo | `--inspect` sobre `~/Libros/*.epub` |
| capítulos vacíos | 0 | ídem (M1.1 corrigió SVG y nav en spine) |
| imágenes resueltas | 2279/2279 | ídem, incluye `<svg><image>` raster |
| biblioteca (medición) | 43 libros, 41 portadas (95,3 %) | `--scan ~/Libros` (2026-11-06) |
| cierre M2 | 50 EPUB (43 reales + 7 copias): 1.65 s; 48/50 portadas | `--scan` en corpus temporal |
| capturas | 4 en `docs/img/` (vacía, llena, búsqueda, error) | `--shot --screen library` |
| escaneo | 1.57 s primera vez; 0.02 s con caché | `/usr/bin/time` + `--scan` |
| `covers/` | 4,2 MB (~98 KB/libro) | `du -sh` |
| tests | 44 unit + 8 integración | `cargo test` |
| clippy | 0 warnings | `cargo clippy --all-targets -- -D warnings` |
| binario `dist` | 5.516.128 B | `cargo build --profile dist` (2026-11-06) |
| cambio de una línea | 2.7–6 s según módulo | `cargo build --profile fast` incremental |
| fixtures | `mini.epub` 2.477 B; `edge.epub` 1.854 B | `scripts/make_fixture.py` |

La medición de biblioteca se hizo con 43 EPUB. Después `~/Libros` quedó en 40
(limpieza manual de duplicados); hoy la biblioteca tiene 40 entradas y 40
portadas. Los números de 43/41 siguen siendo válidos para el corpus de prueba.

El corpus cambió desde M1 (antes 44 libros/2168 imágenes): las métricas viejas
no se comparan directamente. Para reproducir, usar `--inspect` y anotar la fecha.
El 1.3 s de ADR-0006 era la referencia de M1; con el parser más grande, un cambio
en `xhtml.rs` cuesta ~6 s y en `app.rs`/`config.rs` ~2.7–3.5 s.

Advertencia de entorno: el disco sigue trabajando al límite (95% usado). Si
aparece un error de linker raro (`errno=28`), revisar `df -h` antes que el
código.

## Mapa de módulos

```
src/lib.rs     API pública (los tests de integración dependen de ella)
src/main.rs    arranque, flags --inspect / --scan / --shot / --resume / --chapter / --panel
src/app.rs     ventana, biblioteca, lector, barra, scroll, paneles, teclado
src/epub.rs    zip → container.xml → OPF → spine + índice + portada declarada (con #ancla)
src/xhtml.rs   XHTML/HTML → Vec<Block> (entidades, whitespace, pre/tablas, SVG raster, anclas)
src/library.rs índice de carpetas, escaneo incremental, BookEntry
src/covers.rs  miniatura de portada (sin upscaling), hash FNV y caché
src/store.rs   persistencia JSON atómica y backup de archivos corruptos
src/theme.rs   presets de color
src/config.rs  settings en config.json (progreso legado, ya migrado a library.json)
src/fonts.rs   familias Literata en egui
```

## Cómo verificar sin ojos humanos

La app sabe autofotografiarse: `--shot ruta.png` renderiza N frames, guarda el
PNG por `ViewportCommand::Screenshot` y cierra. No depende de permisos de
pantalla. Combinaciones útiles:

```bash
# biblioteca: índice + portadas sin abrir la UI
cargo run --profile fast -- --scan "$HOME/Libros"

# capturas: sin path = biblioteca; con path = lector
cargo run --profile fast -- --shot /tmp/biblioteca.png --screen library
cargo run --profile fast -- --shot /tmp/busqueda.png --screen library --search flores
cargo run --profile fast -- --shot /tmp/otra.png --library-path /tmp/library.json
cargo run --profile fast -- --shot /tmp/libro.png --chapter 13 libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel toc libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel settings libro.epub

# diagnóstico de parser
cargo run --profile fast -- --inspect tests/fixtures/edge.epub

# abrir el último libro (el arranque normal es la biblioteca)
cargo run --profile fast -- --resume
```

Para probar sin tocar la configuración real, compilar y ejecutar el binario con un
`HOME` falso (no usar `cargo run` con `HOME` falso: rustup pierde su toolchain):

```bash
cargo build --profile fast
HOME=$(mktemp -d) ./target/fast/lector --shot /tmp/p.png libro.epub
```

Para leer una captura: sí, hay que mirarla (herramienta de visión). No alcanza
con que el build pase: dos bugs reales (entidades `&quot;` en el OPF, aviso
flotante tapando texto) solo aparecieron así.

## Invariantes que no se rompen

1. **`Block` es el único contrato** entre parseo y render. Agregar un `Kind`
   nuevo es la forma correcta de agregar capacidad visual.
2. **El EPUB aporta estructura, el lector aporta apariencia.** No se interpreta
   CSS del libro.
3. **Un capítulo nunca puede quedar vacío** si el XHTML tiene texto: para eso
   existe el parser tolerante. El test `ningun_capitulo_queda_vacio` lo fija.
4. **Los recursos se resuelven dentro del zip** con rutas normalizadas
   (`resolve()`); aceptar rutas percent-encoded y `..`.
5. **La UI no bloquea**: parsear un capítulo ocurre al abrirlo, no al arrancar.
6. **El texto no se inventa ni se pega**: el whitespace de frontera entre runs se
   conserva, `<pre>` mantiene saltos/indentación y las entidades conocidas se
   decodifican. Un capítulo con `<svg><image>` raster no queda vacío.
7. **`config.json` se reemplaza atómicamente**: nunca se escribe el archivo final
   a medias; si está corrupto, se guarda una copia y se arranca con defaults.
   Lo mismo vale para `library.json` (`src/store.rs`).
8. **La biblioteca indexa, no copia**: las entradas usan la ruta canónica; si un
   archivo desaparece se marca `missing` y no se borra la entrada.
9. **El progreso de libros indexados vive en `library.json`**; `config.progress`
   solo se conserva para leer archivos viejos y no recibe escrituras nuevas.
10. **Las anclas del EPUB se preservan** en `Block.anchor` y `TocEntry.anchor`;
    nunca se descarta el `#fragmento` al parsear nav/ncx.

## Definición de terminado (por tarea)

- [ ] `cargo test` verde y sin warnings nuevos.
- [ ] `cargo fmt --check` limpio.
- [ ] `cargo clippy --all-targets -- -D warnings` limpio.
- [ ] Si tocó UI: captura con `--shot` revisada.
- [ ] Si tocó parseo: `--inspect` sobre el fixture `edge.epub` y al menos 5 EPUB
      del corpus sin capítulos vacíos nuevos.
- [ ] Si tocó biblioteca: `--scan` sobre un fixture o `~/Libros` y revisar
      `library.json`/`covers/`.
- [ ] Spec actualizada (estado, versión, decisiones tomadas).
- [ ] Commit atómico con mensaje en imperativo.

## Qué NO hacer

- No migrar a egui 0.37 "para estar al día" (ADR-0003).
- No introducir SQLite para la biblioteca sin revisar ADR-0007 (el corpus es de
  cientos, no de millones).
- No copiar ni mover los libros al directorio de datos: la biblioteca indexa
  rutas (ADR-0007).
- No agregar un watcher del sistema en M2; se refresca a mano.
- No implementar paginado por columnas: la decisión de M1 fue scroll continuo.
- No rasterizar SVG vectorial ni agregar `resvg` sin ADR (solo se soporta
  `<svg><image>` raster, ver ADR-0009).
- No volver a `fs::write` para `config.json` ni `library.json`: la garantía es
  atómica (`src/store.rs`).
- No tocar la tipografía ni los temas sin mirar una captura.
- No agregar dependencias por comodidad (cada MB del binario se nota).
