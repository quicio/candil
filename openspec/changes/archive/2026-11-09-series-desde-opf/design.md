## Context

Ver `proposal.md` y la spec. `Book::open` ya lee título/autor/idioma y portada del OPF con roxmltree. `BookEntry` se persiste en `library.json` y el escaneo es incremental por `mtime`/`size`; una entrada sin cambios no se vuelve a abrir. La grilla usa `CardData` y `LibraryOrder`.

## Goals / Non-Goals

**Goals:**

- Extraer serie/posición de Calibre y EPUB3.
- Persistir y mostrar la serie sin romper `library.json` viejo.
- Forzar una relectura única para entradas existentes.
- Ofrecer un orden por serie/posición.

**Non-Goals:**

- Filtro por serie, cabeceras de grupo, colecciones de usuario, sinopsis.
- Duplicados, virtualización o búsqueda full-text.

## Decisions

### 1. Dos variantes de metadatos

- Calibre: `<meta name="calibre:series" content="…"/>` y `calibre:series_index`.
- EPUB3: `<meta property="belongs-to-collection" id="s">…</meta>`, `collection-type=series` refinando `#s`, y `group-position` refinando `#s`.

Se prefiere Calibre; si no existe, se elige la primera colección cuyo `collection-type` sea `series`; si hay una sola colección sin tipo, se usa igual.

**Alternativa descartada:** inferir la serie del nombre del archivo o de la carpeta; frágil y propenso a falsos positivos.

### 2. Campos opcionales y `meta_version`

`Book`, `BookEntry` y `CardData` agregan `series: Option<String>` y `series_index: Option<f32>`. `BookEntry.meta_version` (default 0) se compara con `META_VERSION = 1` en la condición de “sin cambios”; si es menor, se relee el OPF y se preservan progreso, ajustes, portada, `added` y `last_opened`.

**Por qué no bumpear `library.json`:** los campos son opcionales y el default es compatible; un contador por entrada evita reabrir toda la biblioteca en cada arranque y no requiere migración de archivo.

### 3. Visualización

`CardData.series` se arma como `Nombre · N` (entero si la posición es entera, un decimal si no). La línea va debajo del autor en la zona de autor, que pasa de 16 a 34 px; sin serie no se dibuja. La etiqueta es dim/itálica, subordinada al título.

**Alternativa descartada:** una fila nueva separada; agranda la tarjeta y compite con el progreso. **Alternativa descartada:** tooltip solamente; la serie quedaría invisible al escanear la grilla.

### 4. Orden por serie

`LibraryOrder::Series` ordena: libros con serie antes que sin serie; por nombre (collate, sin acentos); por `series_index` ascendente (sin índice al final); por título. Se agrega al combo existente, sin filtro nuevo.

**Alternativa descartada:** cabeceras de grupo con scroll; es una reescritura de la grilla y queda para una fase posterior.

## Risks / Trade-offs

- **Metadatos inconsistentes** → se elige la primera colección de tipo serie; si no hay tipo y hay una sola, se usa. Documentado.
- **Relectura única** → la primera pasada tras actualizar puede tardar un poco más; el escaneo es en hilo y conserva datos.
- **Altura de tarjeta** → +18 px; menos tarjetas por pantalla, pero la serie se ve sin abrir el libro.
- **Sin filtro por serie** → el orden agrupa, pero bibliotecas con muchas series pueden pedir filtro después.
- **Posición decimal** → se muestra con un decimal; el orden usa el `f32` real.

## Migration Plan

Sin migración de `library.json`: los campos nuevos usan defaults. El primer escaneo con `meta_version < 1` relee metadatos y actualiza las entradas. Rollback: los campos se ignoran si la versión anterior abre el archivo.

## Open Questions

Ninguna que cambie el alcance. Filtro por serie, cabeceras de grupo y colecciones manuales quedan para después.
