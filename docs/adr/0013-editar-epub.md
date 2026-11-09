# ADR-0013: Edición de metadatos y portada dentro del EPUB

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** biblioteca, EPUB, integridad de datos

## Contexto

La biblioteca indexa los EPUB en su ubicación original (ADR-0007) y no ofrece
forma de corregir un título/autor/idioma ni de agregar una portada. El usuario
puede tener EPUB sin portada o con metadatos mal cargados y hoy solo puede
corregirlos con Calibre u otra herramienta externa.

Editar un EPUB implica reescribir un ZIP y su OPF. Hacerlo mal puede corromper el
libro, perder entradas o dejar la biblioteca apuntando a un archivo inválido.

## Decisión

Agregar una edición acotada, desde el menú contextual de la tarjeta:

- Campos: **título, autor e idioma** (`dc:title`, `dc:creator`, `dc:language`).
- **Portada:** elegir JPG/PNG/WebP, redimensionar a un máximo de 1600×2400 sin
  agrandar, y agregar/reemplazar el ítem `cover-image` del manifiesto.
- **Backup obligatorio:** copiar el EPUB original a
  `data_dir/backups/<nombre>-<nanos>.epub` antes de reemplazar.
- **Escritura segura:** nuevo ZIP en un temporal del mismo directorio,
  `mimetype` primero y sin compresión, resto de entradas copiadas, OPF y portada
  sustituidos.
- **Validación:** abrir el temporal con `Book::open` y comprobar que la portada
  nueva es accesible antes de tocar el original.
- **Reemplazo:** `rename` atómico en Unix; en Windows `remove + rename` con
  restauración desde el backup si falla.
- **Biblioteca:** actualizar la entrada preservando progreso, ajustes, fechas y
  la clave de anotaciones (la ruta no cambia); regenerar la portada cacheada.

## Alternativas descartadas

- **Guardar siempre una copia nueva:** más seguro, pero duplica biblioteca y
  rompe progreso/anotaciones por la nueva ruta. Se puede ofrecer más adelante.
- **Editar el OPF con expresiones regulares y reescribir solo esa entrada:**
  no es posible sin reescribir el ZIP; el enfoque de copiar entradas es más
  robusto.
- **Agregar un writer XML:** otra dependencia y riesgo de perder prefijos o
  elementos desconocidos. La edición de texto sobre OPF se limita a etiquetas
  Dublin Core y al ítem de portada, con tests sobre fixtures.
- **Herramientas externas (Calibre, epubcheck):** no están garantizadas en
  macOS/Linux/Windows y no deben ser una dependencia de ejecución.

## Consecuencias

- El `dist` pasa de 6.818.144 B a **7.234.320 B** (+416.176 B), todavía por
  debajo del presupuesto de 8 MB.
- Cada edición deja un backup; el usuario puede borrarlos desde el directorio de
  datos. Si el disco está lleno, la copia falla y el EPUB original no se toca.
- No se elimina una portada existente ni se editan series, editorial, fecha o
  descripción; quedan para una etapa posterior.
- La validación cubre apertura y portada, no una validación EPUB completa
  (epubcheck). El riesgo residual se acota con el backup.
- Tests con `mini.epub` (portada existente) y `edge.epub` (sin portada) cubren
  agregar, reemplazar, metadatos, `mimetype` primero y backup.
