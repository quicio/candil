---
id: spec-35
title: Preferencias por libro y progreso estable
status: draft
version: 1.0.0
updated: 2026-11-03
milestone: M3
depends_on: [spec-10, adr-0005, spec-20]
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

## Decisión propuesta

1. **Progreso en fracción**: guardar `{ chapter, ratio }` con
   `ratio = offset / max_scroll` (0..1) **y** en qué bloque está el tope del
   viewport. Al restaurar: `offset = ratio * max_scroll` del nuevo layout. El
   ratio sobrevive cambios de fuente y de ventana; el ancla de bloque lo hace
   exacto.
2. **Overrides por libro**: `Settings` opcional por entrada de biblioteca
   (`entry.settings: Option<Settings>`). "Aa" en el lector gana un checkbox
   "solo para este libro"; al desmarcarlo, se borra el override.
3. Precedencia: `entry.settings` → `config.settings` → default.

## Tareas

### T1 — Progreso en fracción
- Cambiar `Progress` a `{ chapter: usize, ratio: f32, anchor: Option<usize> }`
  (el `anchor` es el índice de bloque, no el `id`: sobrevive a cambios de HTML).
- Migración: `offset` viejo → `ratio = offset / max_scroll` no es recuperable sin
  layout; aceptar `ratio = 0` y documentarlo (pérdida única).
- Tests de serialización y de compatibilidad con el JSON viejo.

### T2 — Restaurar con ancla de bloque
- Registrar en cada frame `bloque_visible_superior` (el primer bloque cuyo `y +
  alto > scroll`), guardarlo en `Progress.anchor`.
- Al abrir: `scroll = y_del_bloque[anchor]` si existe, si no `ratio * max_scroll`.

### T3 — Overrides por libro
- `Settings` como `#[serde(default)] Option<Settings>` en `BookEntry`.
- UI: checkbox en el panel Tipografía; indicador discreto en la barra cuando el
  libro tiene ajustes propios.
- Tests: libros A y B con overrides distintos no se pisan.

### T4 — Verificación
- Test manual: cambiar tamaño de fuente en el medio de un capítulo, cerrar,
  reabrir → misma frase arriba.
- Captura con un libro de tema propio sobre fondo del tema global (debe verse el
  propio).

## Criterio de salida

1. Cambiar fuente/interlineado/ancho y reabrir conserva el punto de lectura
   (± 2 líneas).
2. Libros distintos pueden tener temas distintos simultáneamente.
3. `Progress` viejo en JSON no rompe el arranque.

## Riesgos

- Un capítulo muy corto (menos de un viewport): `max_scroll == 0` → `ratio` no
  aplica; usar siempre `anchor` cuando exista.
- El índice de bloque cambia si el parser cambia (un `Kind` nuevo puede partir un
  bloque). Aceptable: se degrada a `ratio`.
