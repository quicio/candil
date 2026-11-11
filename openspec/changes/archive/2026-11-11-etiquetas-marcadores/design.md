## Context

`Bookmark.label` existe desde ADR-0024, pero `add_bookmark_here` siempre pasaba
`String::new()` y el panel solo mostraba el título del capítulo. No había forma
de nombrar ni renombrar una posición.

## Goals / Non-Goals

**Goals:**

- Permitir nombrar un marcador al crearlo o después.
- No cambiar el gesto rápido `B`.
- Mantener el comportamiento actual si la etiqueta queda vacía.

**Non-Goals:**

- Etiquetas obligatorias, filtros por etiqueta o marcadores entre libros.
- Cambios de esquema o migraciones.

## Decisions

### 1. Campo opcional al crear

El panel muestra `Etiqueta opcional` y un `TextEdit`; el botón **Añadir marcador
aquí** usa ese texto y luego limpia el campo. `B` sigue llamando a
`add_bookmark_here` con el campo vacío.

### 2. Edición en el mismo panel

Cada fila tiene **Editar**. Al activarlo, la parte superior del panel cambia a
`Renombrar marcador` con el texto actual y **Guardar/Cancelar**. No se abre otra
ventana ni se agrega un modal.

### 3. Fallback de presentación

Si `label` está vacío o solo tiene espacios, la fila muestra el título del
capítulo y el porcentaje, como antes. `BookEntry::set_bookmark_label` recorta el
texto.

### 4. Persistencia

Se reutiliza `library.json`; el escaneo y `relink` ya copian `bookmarks`. No hay
migración ni archivo nuevo.

## Risks / Trade-offs

- **Panel más alto** → el campo de etiqueta ocupa una fila; el listado conserva
  su scroll.
- **Etiqueta larga** → se trunca a 30 caracteres en la fila; el texto completo
  queda en `library.json`.
- **Edición persistente al cerrar** → si se cierra el panel con una edición
  abierta, al reabrir se retoma; Cancelar la descarta.

## Migration Plan

Sin migración. Rollback: ignorar `label` y volver a mostrar siempre el capítulo.
