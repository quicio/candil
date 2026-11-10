## Context

`library_screen` llamaba a `cards()` con todos los índices visibles y dibujaba
todas las filas dentro de un `ScrollArea`. Las tarjetas se pintan con texturas y
`ui.interact`, así que el costo crece con la biblioteca aunque las filas estén
fuera de pantalla.

## Goals / Non-Goals

**Goals:**

- Reducir tarjetas construidas y dibujadas en bibliotecas grandes.
- Mantener encabezados de serie y acciones de tarjeta.
- No cambiar el comportamiento normal de una biblioteca chica.

**Non-Goals:**

- Reemplazar el scroll por paginado.
- Indexar texto de libros.
- Hacer focusables tarjetas fuera del viewport.

## Decisions

### 1. Umbral 240

Por debajo de 240 libros se usa el camino existente, con todas las tarjetas
focusables. A partir de 240 se virtualiza. Es un umbral alto: el usuario normal
no cambia de comportamiento y la biblioteca grande evita el trabajo extra.

### 2. Layout puro antes de dibujar

`build_grid_items` recibe la serie de cada posición visible y las columnas.
Devuelve `Header { label, count }` o `Row { start, end }`, con alturas fijas
(`GRID_HEADER_H`, `CARD_H + GRID_ROW_GAP`). El total incluye padding superior e
inferior.

`ScrollArea::show_viewport` calcula qué ítems intersectan el viewport (con una
fila de margen) y solo para las filas visibles llama a `cards()`.

### 3. Orden Serie

Los encabezados se construyen a partir de la serie normalizada y se agrupan sin
distinguir mayúsculas. Los libros sin serie comparten el grupo **Sin serie**,
igual que en el render normal.

### 4. Foco

Con virtualización, el foco de teclado recorre las tarjetas renderizadas. Es el
trade-off deliberado para no construir cientos de widgets; el umbral alto hace
que solo afecte a bibliotecas muy grandes. La búsqueda sigue siendo la vía
principal para llegar a un libro lejano.

## Risks / Trade-offs

- **Foco fuera del viewport** → no existe hasta scrollear; documentado.
- **Alturas fijas** → las tarjetas ya tienen `CARD_H` fijo; los encabezados usan
  una altura constante.
- **Grupos con nombres largos** → el encabezado se trunca a 60 caracteres.
- **Costo de `cards()` por fila** → para pocas filas visibles es menor que
  construir toda la biblioteca; el cálculo de duplicados sigue siendo O(n) por
  fila, aceptable con cientos.

## Migration Plan

Sin migración ni datos nuevos. Rollback: quitar la rama `virtualize` y volver al
render completo.
