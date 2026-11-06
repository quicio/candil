---
id: spec-35
title: Preferencias por libro y progreso estable
status: implementada
version: 1.1.0
updated: 2026-11-06
milestone: M3
depends_on: [spec-10, adr-0005, spec-20, adr-0010]
---

# Spec 35 — Preferencias por libro y progreso estable

## Problema 1 — El offset se descoloca

`Progress.offset` está en píxeles. Si el usuario cambia tamaño de fuente,
interlineado, ancho de columna o el tamaño de la ventana, el mismo píxel es otro
texto: al reabrir se cae en un lugar distinto. Con `line_height` 1.55 a 1.9 el
error es de párrafos.

## Problema 2 — Un tema global no sirve para todo

Novelas de noche, manuales técnicos de día. Hoy el tema es global; cambiarlo por
libro obliga a ir a ajustes cada vez.

## Decisión

1. **Progreso en fracción**: guardar `{ chapter, ratio, anchor }` con
   `ratio = offset / max_scroll` (0..1) **y** el índice del bloque visible
   superior. Al restaurar manda el ancla: `offset = y_del_bloque[anchor]`, con el
   ratio como ajuste **dentro** de ese bloque. Así el punto sobrevive cambios de
   fuente, ventana y ancho de columna sin derivar al reabrir.
2. **Overrides por libro**: `Settings` opcional por entrada de biblioteca
   (`entry.settings: Option<Settings>`). "Aa" en el lector gana un checkbox
   "Solo para este libro"; al desmarcarlo, se borra el override.
3. Precedencia: `entry.settings` → `config.settings` → default.

## Implementación (2026-11-06)

### Progreso

- `Progress { chapter, ratio, anchor }`. Para leer JSON de M2 se agrega
  `legacy_offset` con `#[serde(alias = "offset", skip_serializing_if = "Option::is_none")]`:
  el offset viejo se usa **una vez** al abrir y se descarta al volver a guardar.
  No se pierde la posición ni se fuerza `ratio = 0` como preveía el borrador.
- `anchor` es el **índice del bloque** cuyo tope queda en o por encima del
  scroll (`top_block`), no el `id` del EPUB: sobrevive a cambios de HTML y de
  parser. Si el índice deja de existir, se degrada al ratio o al offset legado.
- Al restaurar, `restore_target` recorta el ratio estimado (`ratio * max_scroll`)
  al rango `[y_anchor, y_bloque_siguiente]`. Si el layout no cambió, el ratio cae
  exacto en la posición original; si cambió, nunca se aleja más de un bloque.
- **Normalización de coordenadas:** `ui.cursor()` dentro de `ScrollArea` devuelve
  coordenadas de pantalla. Las posiciones se convierten a coordenadas del
  contenido con `y + state.offset.y - inner_rect.top()` antes de persistir o
  saltar. Sin esto el ancla se corrompía en cuanto el scroll era > 0.
- `+/-` de la barra modifica el override del libro si existe; si no, el global.

### Preferencias por libro

- `BookEntry.settings: Option<Settings>` con `skip_serializing_if`, preservado
  por `scan_folder` en los caminos "sin cambios" y "actualizado".
- Panel Tipografía: checkbox "Solo para este libro", texto de estado y el tema
  efectivo del libro. La barra muestra `Aa·` cuando hay override.
- El tema se resuelve por frame desde `effective_settings`, así que dos libros
  pueden verse con temas distintos sin tocar el global.

### Tests

- `progress_viejo_con_offset_se_migra_al_leer` (serde del JSON viejo).
- `settings_por_libro_tienen_precedencia` y `escaneo_conserva_settings_por_libro`.
- `ancla_no_deriva_al_reabrir`, `ratio_ajusta_dentro_del_ancla` y
  `sin_ancla_cae_a_ratio_o_al_offset_viejo` (matemática de restore).
- Total al cerrar: **52 unit + 8 integración**, clippy sin warnings.

## Tareas

### T1 — Progreso en fracción (hecha)
- `Progress` con `chapter`, `ratio` y `anchor`; compatibilidad con `offset`.
- Tests de serialización y de compatibilidad con el JSON viejo.

### T2 — Restaurar con ancla de bloque (hecha)
- Registrar en cada frame el bloque visible superior y guardarlo.
- Al abrir: ancla primero, ratio recortado al bloque, offset legado como último
  recurso.

### T3 — Overrides por libro (hecha)
- `Settings` opcional en `BookEntry`.
- Checkbox en Tipografía e indicador `Aa·` en la barra.
- Tests de precedencia y de preservación al escanear.

### T4 — Verificación (hecha)
- Captura con `--shot` de un libro con tema propio sobre el tema global: se ve
  Noche con `Aa·`, slider en 17 y checkbox marcado.
- Mismo ancla (bloque 12 de *El mito de Sísifo*, cap. 9) con fuente 27 y 17:
  la primera línea visible es la misma; `library.json` conserva `anchor: 12`.
- Migración de `{chapter, offset}` cubierta por test unitario.

## Criterio de salida

1. Cambiar fuente/interlineado/ancho y reabrir conserva el punto de lectura
   (± 2 líneas). **Cumplido** con ancla + ratio intra-bloque.
2. Libros distintos pueden tener temas distintos simultáneamente. **Cumplido**;
   el global queda intacto.
3. `Progress` viejo en JSON no rompe el arranque. **Cumplido**; se migra al
   leer y se reescribe en el formato nuevo.

## Riesgos

- Un capítulo muy corto (menos de un viewport): `max_scroll == 0` → el ratio no
  aplica; el ancla se recorta al bloque y el scroll queda en 0.
- El índice de bloque cambia si el parser cambia (un `Kind` nuevo puede partir un
  bloque). Aceptable: se degrada a ratio y, tras la primera reapertura, se
  estabiliza con el ancla nueva.
- El checkbox crea un override explícito aunque los valores coincidan con el
  global; es intencional para que el libro quede "fijado" a su tema.
