---
id: spec-45
title: Citas, destacados y notas
status: implementada
version: 1.0.0
updated: 2026-11-06
milestone: M4
depends_on: [spec-30, spec-35, spec-20]
---

# Spec 45 — Citas, destacados y notas

> **Alcance MVP definido el 2026-11-06.** Se implementa **después de M3** porque
> las anclas y el progreso por fracción hacen que los destacados sobrevivan
> mejor a cambios de layout y de parser.

## Objetivo

Poder seleccionar una frase del libro, destacarla y opcionalmente dejarle una
nota; después ver un índice de los destacados/notas del libro y saltar al
fragmento exacto.

## Alcance MVP

Entra:

1. Selección de una frase **dentro de un párrafo** (no cruza bloques).
2. Barra flotante al soltar: **Destacar** y **Nota**.
3. Un único color de destacado (el acento del tema). La nota es opcional y vive
   sobre el mismo fragmento.
4. Persistencia en `annotations.json` (archivo propio, escritura atómica).
5. Panel **“Destacados y notas”** del libro abierto: cita, nota, capítulo,
   salto al fragmento, editar nota y borrar.
6. Los offsets son de texto, no de píxeles: cambiar fuente, interlineado o ancho
   de ventana no mueve el destacado. Si el bloque cambió, se reancla buscando la
   cita.

No entra:

- Índice global de todos los libros (segunda etapa).
- Varios colores o subrayado tipo Kindle.
- Selección que cruce párrafos.
- Exportación a Markdown.
- Notas sueltas sin fragmento.
- Sincronización entre dispositivos.

## Modelo

```rust
// src/annotations.rs
#[derive(Serialize, Deserialize, Clone)]
pub struct Annotation {
    pub id: u64,
    pub book: String,        // ruta canónica
    pub chapter: usize,      // índice del spine
    pub block: usize,        // índice en Vec<Block>
    pub start: usize,        // offset en caracteres dentro de Block::text()
    pub end: usize,
    pub quote: String,       // texto seleccionado para mostrar/reanclar
    pub note: String,        // vacío = solo destacado
    pub color: u8,           // 0 en MVP
    pub created: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Annotations {
    pub schema_version: u32, // 1
    pub items: Vec<Annotation>,
}
```

Rutas: `dirs::config_dir()/lector/annotations.json`, usando `src/store.rs`
(temporal + `rename` + backup de corruptos). No se mezcla con `library.json`
para no hinchar el índice de la estantería.

## UI

```
selección de texto
      │
      ▼
┌─────────────────┐
│ Destacar │ Nota │  barra flotante cerca de la selección
└─────────────────┘

barra del lector:  ☰  Aa  ✎  …
                        │
                        ▼
             “Destacados y notas”
             ┌──────────────────────────────┐
             │ “cita seleccionada…”         │
             │ nota (si tiene)              │
             │ Capítulo 7        [Ir] [×]   │
             └──────────────────────────────┘
```

- `✎` en la barra del lector y atajo `D` abren el panel.
- Click en una entrada carga el capítulo y hace scroll al bloque.
- `Nota` abre una ventana con `TextEdit` multilínea; guardar crea el destacado
  con nota. Si la nota queda vacía, es un destacado normal.
- Borrar quita la anotación; no toca el EPUB.

## Selección y render

- El lector hoy usa `Label::selectable(false)`. Hay que habilitar selección en
  los bloques de texto sin perder tipografía, justificación, interlineado ni
  espaciado.
- La opción preferida es un `TextEdit` de solo lectura por bloque de texto
  (buffer `&str` + `layouter` con el mismo `LayoutJob`), que expone
  `cursor_range` y permite mapear la selección a offsets del bloque.
- El resaltado se pinta con `TextFormat::background` en las secciones del
  `LayoutJob` que caen dentro de `[start, end)`.
- Los bloques que no son texto (imagen, regla) siguen igual.
- Si el `block` guardado ya no contiene la cita en `[start, end)`, se busca la
  cita en el capítulo y se usa la primera coincidencia; si no aparece en el
  capítulo actual, el panel la marca como “no encontrada en este capítulo”.

## Implementación (2026-11-06)

- **Modelo**: `src/annotations.rs` con `Annotation`/`Annotations`, `schema_version: 1`,
  `load/save` atómicos vía `src/store.rs`, `for_book`, `sorted_for_book`, `add`
  (id = max + 1), `remove`, `update_note` y `get`/`get_mut`.
- **Selección**: cada bloque de texto se dibuja con un `TextEdit` multilínea
  **inmutable** (`&mut &str`, sin marco, margen cero, una fila mínima) y un
  `layouter` que reutiliza el `LayoutJob` de siempre. Así se conservan
  justificación, interlineado, espaciado y estilos inline. Al soltar el mouse,
  `cursor_range` se convierte a offsets de `Block::text()`
  (`selection_range`), descontando el prefijo de viñeta/numeración; la cita se
  extrae del bloque.
- **Resaltado**: `LayoutJob` se parte en los bordes de cada anotación y las
  secciones internas llevan `TextFormat::background` con el acento al 33 % de
  alfa. El resaltado se calcula por frame y no toca el EPUB.
- **Reanclaje**: si el bloque guardado cambió, `resolve_annotation` busca la cita
  primero en ese bloque y después en todo el capítulo. Si no aparece, la
  anotación no se dibuja (el panel la sigue mostrando con su cita).
- **UI**: barra flotante `Destacar`/`Nota` sobre la selección; `H` y `N`;
  ventana de nota multilínea (crear o editar); panel “Destacados y notas” (`D`)
  con cita, nota, capítulo, `Ir`, `Editar` y `×`. `Esc` sale de la selección o
  cierra la ventana activa sin volver a la biblioteca.
- **Salto**: `(chapter, block, quote)`; si el capítulo cambia se carga y el
  bloque se ubica con `locate_quote`, luego se ajusta el scroll con las
  posiciones registradas al dibujar (`pending_block_jump`).
- **Persistencia**: `annotations.json` separado de `library.json`, escritura
  atómica y backup de corruptos. Borrar una anotación no toca el EPUB.
- **Desviación menor**: el botón de la barra dice `Notas` en lugar del glifo
  `✎`, porque Literata y la fuente de fallback de la app no traen ese carácter y
  se veía un cuadrado. La función y el atajo son los de la spec.
- **Dev**: `--panel notes` y `--panel note` abren el panel y el editor para las
  capturas; `--shot` sigue siendo el mecanismo de verificación visual.

## Tareas

### T1 — Modelo y persistencia (hecha)
- Crear `src/annotations.rs`, `pub mod annotations;`.
- `load/save`, `for_book`, `add`, `remove`, `update_note`, `id` único.
- Tests: roundtrip, JSON corrupto con backup, agregar/borrar.

### T2 — Selección de texto (hecha)
- Renderizar bloques de texto con selección sin romper la tipografía.
- Mapear `cursor_range` a `(block, start, end, quote)`, descontando el prefijo
  de viñeta/numeración si existe.
- Test visual: seleccionar una frase en `flores-del-mal` y en un capítulo normal;
  medir que el capítulo gigante no se vuelva inusable.

### T3 — Destacar (hecha)
- Barra flotante `Destacar` / `Nota` cuando hay selección no vacía.
- Crear `Annotation` con `color: 0`, guardar y repintar el `LayoutJob` con fondo.
- Atajo `H` para destacar la selección.

### T4 — Nota (hecha)
- Ventana de nota multilínea; guardar/editar.
- `N` abre la nota de la selección.
- Una anotación con nota se ve igual que un destacado; el panel muestra la nota.

### T5 — Panel del libro (hecha)
- Ventana “Destacados y notas” (`D`/`✎`) con lista ordenada por capítulo/bloque.
- Salto: `(chapter, block)` + posiciones registradas al dibujar; `pending_jump`
  y `request_repaint`.
- Borrar y editar nota desde el panel.
- Estados: sin anotaciones (“Todavía no destacaste nada.”).

### T6 — Verificación y docs (hecha)
- Tests de modelo (`annotations.rs`), reanclaje por cita, selección con prefijo y
  `locate_quote` en `app.rs`. Total: **64 unit + 8 integración**.
- Capturas en `docs/img/reader-highlight.jpg`, `reader-notes.jpg` y
  `reader-note.jpg` (lector con destacado, panel y editor).
- Rendimiento del caso grande: `flores-del-mal` (1895 bloques en un capítulo),
  `--shot` de 12 frames en 0.52 s; no hizo falta el modo “Anotar”.
- README/HANDOFF/ARCHITECTURE/spec actualizados.

## Criterio de salida

1. Seleccionar una frase, destacarla, cerrar y reabrir: el destacado sigue en el
   mismo texto. **Cumplido**: offsets de caracteres + cita; se reancla si el
   bloque cambió.
2. Cambiar tamaño de fuente y ancho de ventana: el destacado no se mueve de
   frase. **Cumplido**: verificado a 17 y 27 con el mismo `Block` y cita.
3. Agregar una nota, verla en el panel y saltar al fragmento. **Cumplido**:
   `NoteEditor` crea/edita y el panel resuelve `pending_block_jump`.
4. Borrar una anotación no deja restos visuales. **Cumplido**: se quita del
   `Vec`, se guarda y el resaltado se recalcula por frame.
5. `cargo test` verde y capturas revisadas. **Cumplido**; clippy sin warnings.

## Riesgos

| riesgo | mitigación |
|---|---|
| Selección por bloque no cruza párrafos | aceptado en MVP; se puede extender después |
| `TextEdit` por bloque degrada el capítulo gigante | medir `flores-del-mal`; si hace falta, activar selección solo en modo “Anotar” |
| Índice de bloque cambia si cambia el parser | reanclar por `quote`; M3 aporta anclas de fragmento |
| Muchas anotaciones hinchan el JSON | archivo aparte, escritura atómica; revisar si supera ~5.000 |
| Nota sobre selección vacía | deshabilitar `Destacar`/`Nota` si no hay selección |
