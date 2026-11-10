## Context

`search.rs` construye un índice efímero por libro con capítulos y bloques
normalizados. La biblioteca no puede construir todos los índices al arrancar: el
texto crece con cada EPUB y el disco está limitado. La búsqueda de metadatos ya
filtra la grilla en cada tecla.

## Goals / Non-Goals

**Goals:**

- Buscar una frase en el contenido de la biblioteca sin bloquear la UI.
- No persistir texto ni índices en disco.
- Llegar al pasaje encontrado con el mismo resaltado de la búsqueda interna.

**Non-Goals:**

- Reemplazar el índice efímero por uno persistente.
- Búsqueda instantánea en cada tecla o resultados entre sesiones.
- Indexar notas, metadatos externos o contenido de EPUB rotos.

## Decisions

### 1. Búsqueda bajo demanda, un libro a la vez

Al presionar Enter, el hilo recorre los libros disponibles, abre cada EPUB,
construye su `SearchIndex` con `search::build`, busca y descarta el índice. El
pico de memoria es el de un libro, no el de toda la biblioteca.

**Por qué no un índice global:** ocuparía decenas o cientos de MB y, si se
persistiera, competiría con el poco disco libre. La búsqueda es una acción
explícita y poco frecuente.

### 2. Límites explícitos

- `CONTENT_SEARCH_MAX_HITS = 100` global.
- `CONTENT_SEARCH_PER_BOOK = 20` para que un libro no monopolice la lista.
- `CONTENT_SEARCH_MAX_BOOKS = 120` para acotar el peor caso.
- Mínimo 3 caracteres, igual que la búsqueda interna.

Si se corta por cualquier límite, el panel lo dice.

### 3. Hilo y cancelación

El worker envía `Progress`, `Hits` y `Done` por `mpsc`; la UI drena el canal en
cada frame y pide repaint mientras está ocupado. `AtomicBool` permite cancelar
entre libros (no dentro de un EPUB, para no fragmentar el parser).

### 4. Salto al pasaje

El resultado guarda ruta, capítulo, bloque y offsets. Al elegirlo se abre el
libro, se carga el capítulo y se reutilizan `pending_block_jump`,
`pending_search_jump` y `search_highlight`, la misma ruta que un resultado de la
búsqueda interna.

### 5. Sin cambios de datos

No se toca `library.json`, `annotations.json` ni se crea un archivo de índice.
El respaldo y el escaneo quedan igual.

## Risks / Trade-offs

- **Búsqueda lenta en bibliotecas grandes** → progreso, cancelación y tope de 120
  libros; no congela la UI.
- **Sin resultados instantáneos** → es una acción explícita con Enter/botón, no
  cada tecla.
- **Un libro muy grande** → el worker puede tardar dentro de ese libro; el
  cancelar se aplica al terminar el actual.
- **Resultados limitados** → se avisa cuando hay más de los mostrados.

## Migration Plan

Sin migración. Rollback: quitar el estado, el worker y el panel; `search.rs` sigue
igual para la búsqueda interna.
