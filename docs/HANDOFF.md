---
id: handoff
status: vigente
version: 1.0.0
updated: 2026-11-03
---

# Handoff — estado del proyecto y cómo seguir

## Qué es

Lector de EPUB de escritorio en Rust: una columna, scroll continuo, tipografía
propia, temas prearmados. egui/eframe para la ventana, parser XHTML propio para
el contenido. Sin webview, sin motor CSS.

**M1 (lector) está terminado y verificado.** Lo que sigue es M2 (biblioteca).

## Estado medido (no estimado)

| métrica | valor | cómo se midió |
|---|---|---|
| corpus real | 44 EPUB, 0 errores de parseo | `--inspect` sobre `~/Downloads/*.epub` |
| imágenes resueltas | 2168/2168 | ídem |
| tests | 14 unit + 6 integración | `cargo test` |
| binario `dist` | 5.298.032 B | `cargo build --profile dist` |
| cambio de una línea | 1.3 s | `cargo build --profile fast` incremental |
| fixture propio | 2.470 B, 3 capítulos | `tests/fixtures/mini.epub` |

Advertencia de entorno: el disco de la máquina estaba al 100% y un link falló
con `errno=28`. Si aparece un error de linker raro, revisar `df -h` antes que el
código.

## Mapa de módulos

```
src/lib.rs     API pública (los tests de integración dependen de ella)
src/main.rs    arranque, flags --inspect / --shot / --chapter / --panel
src/app.rs     ventana, barra, scroll, paneles, entrada de teclado
src/epub.rs    zip → container.xml → OPF → spine + índice
src/xhtml.rs   XHTML/HTML → Vec<Block> (parser estricto + tolerante)
src/theme.rs   presets de color
src/config.rs  settings y progreso en JSON
src/fonts.rs   familias Literata en egui
```

## Cómo verificar sin ojos humanos

La app sabe autofotografiarse: `--shot ruta.png` renderiza N frames, guarda el
PNG por `ViewportCommand::Screenshot` y cierra. No depende de permisos de
pantalla. Combinaciones útiles:

```bash
cargo run --profile fast -- --shot /tmp/p.png --chapter 13 libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel toc libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel settings libro.epub
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

## Definición de terminado (por tarea)

- [ ] `cargo test` verde y sin warnings nuevos.
- [ ] `cargo fmt --check` limpio.
- [ ] Si tocó UI: captura con `--shot` revisada.
- [ ] Si tocó parseo: `--inspect` sobre el fixture y al menos 5 EPUB del corpus
      sin capítulos vacíos nuevos.
- [ ] Spec actualizada (estado, versión, decisiones tomadas).
- [ ] Commit atómico con mensaje en imperativo.

## Qué NO hacer

- No migrar a egui 0.37 "para estar al día" (ADR-0003).
- No introducir SQLite para la biblioteca sin revisar ADR-0007 (el corpus es de
  cientos, no de millones).
- No implementar paginado por columnas: la decisión de M1 fue scroll continuo.
- No tocar la tipografía ni los temas sin mirar una captura.
- No agregar dependencias por comodidad (cada MB del binario se nota).
