## Context

`BookEntry` guarda `title`, `author` y `path`. La biblioteca no guarda hashes de
contenido y el escaneo ya es incremental por tamaño y `mtime`. El filtro existente
tiene estados de lectura y series.

## Goals / Non-Goals

**Goals:**

- Detectar el caso real: misma obra indexada dos veces, aun con distinto nombre
  de archivo.
- Revisar los candidatos sin salir de la biblioteca y sin borrar nada.
- No agregar dependencias, hashes ni campos a `library.json`.

**Non-Goals:**

- Duplicados exactos por contenido, fusión de metadatos o elección automática.
- Ignorar ediciones legítimas con el mismo título y autor.

## Decisions

### 1. Clave conservadora: título + autor normalizados

`duplicate_key` usa `collate` sobre título y autor (minúsculas y acentos
plegados). Si el título queda vacío, no hay clave. La clave se calcula en memoria
en cada vista; el corpus es de cientos de libros.

**Por qué:** cubre copias con otro nombre de archivo sin leer el EPUB ni hashear
el archivo. **Alternativa descartada:** hash completo en cada escaneo, más lento y
con más estado que mantener.

### 2. “Posible duplicado”, no “duplicado”

La insignia dice **Posible duplicado ×N** y el tooltip explica que el criterio es
título y autor. Dos ediciones distintas pueden compartir ambos datos; Candil solo
señala, no decide.

### 3. Filtro y acción explícita

`LibraryFilter::Duplicates` muestra solo claves con más de una entrada. La
tarjeta conserva el menú contextual existente (`Mostrar en Finder`, `Quitar de la
biblioteca`), así el usuario decide cuál conservar. Nunca se borra el EPUB.

### 4. Banderas de desarrollo

`--filter duplicates` permite capturar el estado sin clics; se suma a `--order` y
`--series`.

## Risks / Trade-offs

- **Falso positivo** → la UI habla de “posible” y no borra nada.
- **Falso negativo** (misma obra con título o autor distintos) → aceptado; se
  necesitaría comparación de contenido o metadatos externos.
- **Costo** → O(n) por vista sobre los metadatos en memoria; despreciable para
  cientos de libros.

## Migration Plan

Sin migración. No se persiste la detección ni el filtro. Rollback: quitar la
variante, la insignia y la clave.
