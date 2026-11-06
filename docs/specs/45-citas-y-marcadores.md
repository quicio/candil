---
id: spec-45
title: Citas, destacados y notas
status: draft
version: 0.2.0
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
  cita en el capítulo y se usa la primera coincidencia; si no aparece, la
  anotación se muestra en el panel como “no encontrada”.

## Tareas

### T1 — Modelo y persistencia
- Crear `src/annotations.rs`, `pub mod annotations;`.
- `load/save`, `for_book`, `add`, `remove`, `update_note`, `id` único.
- Tests: roundtrip, JSON corrupto con backup, agregar/borrar.

### T2 — Selección de texto
- Renderizar bloques de texto con selección sin romper la tipografía.
- Mapear `cursor_range` a `(block, start, end, quote)`, descontando el prefijo
  de viñeta/numeración si existe.
- Test visual: seleccionar una frase en `flores-del-mal` y en un capítulo normal;
  medir que el capítulo gigante no se vuelva inusable.

### T3 — Destacar
- Barra flotante `Destacar` / `Nota` cuando hay selección no vacía.
- Crear `Annotation` con `color: 0`, guardar y repintar el `LayoutJob` con fondo.
- Atajo `H` para destacar la selección.

### T4 — Nota
- Ventana de nota multilínea; guardar/editar.
- `N` abre la nota de la selección.
- Una anotación con nota se ve igual que un destacado; el panel muestra la nota.

### T5 — Panel del libro
- Ventana “Destacados y notas” (`D`/`✎`) con lista ordenada por capítulo/bloque.
- Salto: `(chapter, block)` + posiciones registradas al dibujar; `pending_jump`
  y `request_repaint`.
- Borrar y editar nota desde el panel.
- Estados: sin anotaciones (“Todavía no destacaste nada.”).

### T6 — Verificación y docs
- Tests de modelo, reanclaje por cita y persistencia.
- Captura: lectura con un destacado, panel de notas, nota abierta.
- Actualizar README/HANDOFF/spec 45 al implementar.

## Criterio de salida

1. Seleccionar una frase, destacarla, cerrar y reabrir: el destacado sigue en el
   mismo texto.
2. Cambiar tamaño de fuente y ancho de ventana: el destacado no se mueve de
   frase.
3. Agregar una nota, verla en el panel y saltar al fragmento.
4. Borrar una anotación no deja restos visuales.
5. `cargo test` verde y capturas revisadas.

## Riesgos

| riesgo | mitigación |
|---|---|
| Selección por bloque no cruza párrafos | aceptado en MVP; se puede extender después |
| `TextEdit` por bloque degrada el capítulo gigante | medir `flores-del-mal`; si hace falta, activar selección solo en modo “Anotar” |
| Índice de bloque cambia si cambia el parser | reanclar por `quote`; M3 aporta anclas de fragmento |
| Muchas anotaciones hinchan el JSON | archivo aparte, escritura atómica; revisar si supera ~5.000 |
| Nota sobre selección vacía | deshabilitar `Destacar`/`Nota` si no hay selección |
