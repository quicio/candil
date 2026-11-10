## Context

`BookEntry` ya guarda `progress: Progress`, `settings` y fechas. Las anotaciones
son rangos con cita y color; no sirven como marcador de posición. El panel de
lectura tiene accesos para índice, tipografía, notas, búsqueda, Ir a y consulta.

## Goals / Non-Goals

**Goals:**

- Guardar y recuperar posiciones exactas con un gesto rápido.
- Reutilizar `Progress`, la persistencia y el respaldo existentes.
- Mantener los marcadores al reindexar o reasociar el EPUB.

**Non-Goals:**

- Etiquetas personalizadas, carpetas de marcadores o marcadores entre libros.
- Exportación o sincronización.
- Cambiar el formato de `annotations.json`.

## Decisions

### 1. `BookEntry.bookmarks`

Cada marcador guarda `{id, progress, label, created}`. `progress` es el mismo
tipo de la persistencia, así el salto respeta ancla y fracción. `label` queda
vacío en esta fase; la UI muestra capítulo y porcentaje. El campo se omite del
JSON si la lista está vacía.

### 2. Alta y deduplicación

`B` toma `current_progress()` y agrega si no hay otro marcador en el mismo
capítulo con el mismo ancla y menos de 1 % de diferencia (o menos de 1 % de ratio
si no hay ancla). El id es `max(id)+1`, local al libro.

### 3. Panel y salto

`Mayús+B` o el botón **Marcador** abren/cierran un panel con la lista ordenada.
Saltar llama a `load_chapter(chapter, Some(progress))`, la misma restauración que
usa el progreso; no se guarda un `Progress` de regreso porque el marcador no es
una vista previa. Eliminar es inmediato y guarda `library.json`.

### 4. Preservación

El escaneo incremental y `relink` copian `bookmarks` igual que `progress` y
`settings`. El respaldo no cambia: `library.json` ya está incluido.

## Risks / Trade-offs

- **Crecimiento de `library.json`** → una lista corta por libro; sin límite
  explícito en esta fase.
- **Marcador en posición repetida** → la deduplicación por ancla/ratio evita
  duplicados obvios.
- **Etiqueta vacía** → se muestra el título del capítulo; el campo queda para una
  fase posterior.
- **Salto entre capítulos** → recarga el capítulo y restaura, como el progreso.

## Migration Plan

Sin migración: `bookmarks` usa `#[serde(default)]`. Rollback: ignorar el campo;
los datos quedan en `library.json` sin afectar la lectura.
