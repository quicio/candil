## Context

`start_library_content_search` abría cada EPUB y construía `search::build` en
cada consulta. `SearchIndex` ya es un tipo puro y no depende de la UI. El disco
está limitado; el respaldo no debe crecer.

## Goals / Non-Goals

**Goals:**

- Acelerar búsquedas repetidas sin cambiar el flujo ni la UI.
- Acotar el uso de RAM.
- Invalidar correctamente cuando el EPUB cambia.

**Non-Goals:**

- Persistir índices entre sesiones.
- Cachear resultados por consulta.
- Indexar en segundo plano al arrancar.

## Decisions

### 1. LRU de 64 MB

`SearchCache` guarda `Arc<SearchIndex>`, bytes estimados y una lista ordenada por
uso. `get` mueve al final; `put` evicta desde el frente hasta que entra en
`CONTENT_SEARCH_CACHE_BYTES = 64 MB`. Un índice individual mayor al tope no se
cachea.

### 2. Clave por archivo

`cache_key(path)` usa ruta + `len` + `mtime`. Una edición cambia la clave, así que
la entrada vieja no se usa y se evicta naturalmente.

### 3. Concurrencia

El caché vive en el `Lector` como `Arc<Mutex<SearchCache>>` y se clona al worker.
El `Arc<SearchIndex>` se suelta antes de buscar; el mutex no se mantiene durante el
trabajo pesado.

### 4. Sin persistencia

No se escribe nada en `library.json` ni en un archivo aparte. Al cerrar la app se
libera.

## Risks / Trade-offs

- **RAM** → 64 MB de tope; puede desplazar índices de libros poco usados.
- **Primera búsqueda** → sigue pagando el parseo.
- **Archivos cambiados** → clave nueva; la entrada vieja ocupa hasta que se
  evicta.
- **Índice individual enorme** → no se cachea, se busca en el momento.

## Migration Plan

Sin migración. Rollback: quitar el campo y volver a construir en cada búsqueda.
