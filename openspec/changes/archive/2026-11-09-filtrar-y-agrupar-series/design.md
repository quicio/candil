## Context

`Library::series_names` no existía; el filtro solo tenía estados de lectura
(Todos, Leyendo, Sin empezar, Faltantes) y el orden Serie ordenaba los índices
pero la grilla dibujaba filas continuas. `CardData` ya formateaba la serie para
la tarjeta, pero no conservaba el nombre crudo para agrupar.

## Goals / Non-Goals

**Goals:**

- Aislar una serie o los libros sin serie sin agregar otro control permanente.
- Dar contexto visual al orden por serie.
- Reutilizar el filtro existente y no tocar `library.json`.

**Non-Goals:**

- Editar series, detectar duplicados, virtualizar la grilla o agrupar por autor.
- Cambiar el orden por serie ni la extracción del OPF.

## Decisions

### 1. La serie entra en el filtro existente

`LibraryFilter` gana `Series(String)` y `NoSeries`. El combo de filtro lista los
estados, luego las series detectadas y, si hay libros sin serie, **Sin serie**.
Así no aparece un tercer combo que compita con búsqueda, orden y filtro.

**Por qué:** el filtro ya es el lugar mental para “qué quiero ver”. **Alternativa
descartada:** un combo “Serie” separado, que agrega ruido y no cabe bien a 720 px.

### 2. Nombres de serie normalizados

`Library::series_names` recorta espacios, descarta vacíos, ordena sin distinguir
mayúsculas y elimina duplicados conservando el primer nombre. El filtro compara
el nombre exacto guardado en `BookEntry.series`.

### 3. Encabezados solo en orden Serie

`CardData` conserva `series_name`. Si el orden es Serie y hay al menos una serie,
la grilla agrupa las tarjetas consecutivas y dibuja un encabezado con nombre y
cantidad. Si ningún libro tiene serie, no se muestran encabezados. El grupo sin
serie se llama **Sin serie** y queda al final por el orden existente.

### 4. Banderas de desarrollo

`--order series` y `--series "Nombre"` permiten capturar y verificar los estados
sin depender de clics. Solo aplican al arranque en desarrollo/capturas.

## Risks / Trade-offs

- **Muchas series** → el combo de filtro crece; se acepta porque el corpus es de
  cientos, no de miles, y el combo ya era el lugar de filtros.
- **Nombre largo** → la etiqueta recorta a 22 caracteres con puntos suspensivos;
  el nombre completo sigue visible en la tarjeta.
- **Filtro huérfano** → si una serie desaparece tras un escaneo, la grilla queda
  vacía y el estado vacío ofrece **Ver todos**.

## Migration Plan

Sin migración. `library_filter` es estado de UI y no se persiste; `library.json`
no cambia. Rollback: quitar las variantes y los encabezados.
