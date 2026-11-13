# ADR-0028: Caché en memoria de índices full-text

- **Estado:** aceptado
- **Fecha:** 2026-11-13
- **Área:** biblioteca, búsqueda, rendimiento

## Contexto

La búsqueda full-text abría y parseaba cada EPUB en cada consulta. Eso es
correcto para una búsqueda aislada, pero castiga la segunda búsqueda y cualquier
refinamiento de la consulta. Un índice persistente en disco resolvería la
velocidad, pero el disco está limitado y el texto de todos los libros crece con
la biblioteca.

## Decisión

Mantener una **caché LRU en memoria** de `search::SearchIndex` por libro:

- Clave: ruta + tamaño + mtime (`cache_key`). Si el EPUB cambia, la clave cambia y
  no se reutiliza texto viejo.
- Valor: `Arc<SearchIndex>` para que el worker de búsqueda lo use sin mantener el
  mutex tomado.
- Tope: **64 MB** (`CONTENT_SEARCH_CACHE_BYTES`). Si un índice solo supera el
  tope, no se cachea; si la suma se pasa, se evicta el menos usado.
- Se comparte entre búsquedas con `Arc<Mutex<SearchCache>>`; no se persiste.
- Se pierde al cerrar la app y no entra en el respaldo.

### Flujo

1. El worker calcula la clave del libro.
2. Si está en caché, busca sobre el índice cacheado.
3. Si no, abre el EPUB, construye el índice, lo guarda y busca.
4. El progreso y los resultados siguen saliendo por `mpsc`.

## Alternativas descartadas

- **Índice persistente en disco:** búsquedas más rápidas entre sesiones, pero
  ocupa espacio proporcional a todos los EPUB y hay que invalidarlo.
- **Caché sin tope:** simple, pero una biblioteca grande puede consumir cientos
  de MB de RAM.
- **Cachear resultados por consulta:** no ayuda cuando el usuario cambia una
  palabra o busca otra frase.
- **Reabrir siempre:** el comportamiento anterior, lento en consultas repetidas.

## Consecuencias

- La primera búsqueda de una sesión sigue pagando el parseo; las siguientes
  reutilizan texto y son mucho más rápidas.
- El uso de RAM está acotado a 64 MB.
- No hay archivos nuevos ni migración.
- Tests cubren reutilización y evicción del caché.
