# ADR-0033: Caché local de EPUB en montajes remotos

- **Estado:** aceptado
- **Fecha:** 2026-11-15
- **Área:** biblioteca, lector, búsqueda

## Contexto

La biblioteca puede indexar carpetas de Google Drive montadas con `rclone`. Con
`--vfs-cache-mode writes` las lecturas no se cachean: cada apertura, búsqueda o
salto vuelve a pedir el EPUB completo al montaje remoto. Abrir un libro y
construir su índice de búsqueda se vuelve lento y depende de la red.

La ruta original es la identidad del libro: `library.json`, progreso,
anotaciones, marcadores y study guardan esa ruta. Cambiarla a una copia local
rompería esos vínculos y haría que un movimiento del libro pareciera otro libro.

## Decisión

1. **Caché descartable en `~/.cache/candil/books`.** No vive en el directorio de
   datos, no entra en el respaldo de cinco JSON y se puede borrar sin perder
   nada. El límite es **1 GB** con evicción LRU.
2. **Solo montajes remotos.** En Linux se detectan leyendo `/proc/mounts`
   (FUSE, rclone, sshfs, nfs, cifs y similares). Un EPUB local nunca se copia.
   En otras plataformas la caché queda inactiva hasta que haya detección.
3. **Índice persistente mínimo.** `index.json` guarda ruta canónica, tamaño,
   mtime, nombre de archivo, bytes y último uso. La copia se invalida si cambia
   tamaño o mtime; se escribe con temporal + `rename`.
4. **Copia atómica.** El EPUB se copia a un temporal y se renombra al nombre
   final. Si falla, el original sigue siendo la fuente.
5. **Identidad estable.** `Book::open_from(identity, source)` lee la copia
   cacheada pero conserva `Book.path` con la ruta original. `open_cached_book`
   intenta la caché y cae al original si algo falla; la apertura y la búsqueda
   avanzada usan el mismo camino.
6. **Preparar offline.** Desde el menú de la biblioteca, **Preparar offline**
   copia en un hilo los EPUB remotos visibles (filtro y búsqueda incluidos), con
   progreso y cancelación. No descarga libros marcados como faltantes o con
   error.
7. **Limpieza explícita.** Configuración → Avanzado muestra directorio, cantidad
   y tamaño; **Limpiar caché** borra archivos e índice. No se limpia sola al
   cerrar ni se incluye en exportaciones.
8. **Sin bloqueo de UI.** La copia ocurre en hilos de apertura, búsqueda u
   offline. La UI solo recibe progreso y sigue respondiendo.

## Alternativas descartadas

- **Cambiar `Book.path` a la ruta cacheada:** rompe progreso, anotaciones,
  marcadores y la biblioteca; además confunde libros movidos con libros nuevos.
- **Pedir a rclone `--vfs-cache-mode full`:** es una buena mitigación del
  entorno, pero no controla el límite de Candil, no permite preparar offline
  desde la app y no cubre otros montajes remotos.
- **Caché sin límite:** puede llenar el disco, que es un recurso crítico en este
  proyecto.
- **Índice de búsqueda persistente en la caché:** mayor complejidad de
  invalidación y disco; por ahora alcanza con no releer el EPUB remoto.
