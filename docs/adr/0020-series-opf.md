# ADR-0020: Series y colecciones desde el OPF

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** biblioteca, metadatos, EPUB

## Contexto

Muchos EPUB de Calibre y EPUB3 declaran a qué serie pertenecen y en qué posición.
Candil leía título, autor, idioma y portada, pero descartaba esa información; los
libros de una saga quedaban sueltos y el orden no reflejaba la colección.

`library.json` ya existe en instalaciones previas y no queremos migrar el
archivo a mano ni reabrir toda la biblioteca en cada arranque.

## Decisión

### Lectura

- **Calibre:** `<meta name="calibre:series" content="…"/>` y
  `<meta name="calibre:series_index" content="N"/>`.
- **EPUB3:** `<meta property="belongs-to-collection" id="s">…</meta>`,
  `<meta refines="#s" property="collection-type">series</meta>` y
  `<meta refines="#s" property="group-position">N</meta>`.

Se prefiere Calibre. Si no está, se elige la primera colección con
`collection-type=series`; si hay una sola colección sin tipo, se usa igual.
`series_index` se parsea como `f32` (admite `2.5`).

### Persistencia

`Book` y `BookEntry` agregan `series: Option<String>` y
`series_index: Option<f32>`. `library.json` los guarda con `serde(default)`.
`BookEntry.meta_version` (default 0) se compara con `META_VERSION = 1`; una
entrada vieja se vuelve a leer en el próximo escaneo aunque el `mtime` no cambie.
La relectura conserva `progress`, `settings`, `cover`, `added` y `last_opened`.

### Visualización y orden

La tarjeta muestra `Serie · N` debajo del autor (dim, itálica); sin serie no hay
línea. La zona de autor pasa de 16 a 34 px.

`LibraryOrder::Series` ordena: con serie antes que sin serie, por nombre
normalizado, por posición ascendente (sin posición al final) y por título.

### Alternativas descartadas

- **Inferir series del nombre de archivo/carpeta:** frágil y con falsos positivos.
- **Bumpear `library.json` y migrar todo:** más ruido; los campos opcionales con
  `meta_version` logran lo mismo.
- **Tooltip solamente:** la serie no se ve al recorrer la grilla.
- **Cabeceras de grupo:** es una reescritura de la grilla; queda para después.
- **Filtro por serie ahora:** el orden resuelve el caso principal; el filtro
  puede sumarse luego.

## Consecuencias

- La primera biblioteca que se abra después de actualizar puede tardar un poco
  más en el primer escaneo; es en hilo y solo una vez por entrada.
- Los EPUB sin metadatos de serie no cambian.
- La tarjeta es 18 px más alta; entran menos por pantalla.
- Series con nombres distintos para la misma saga no se unifican; es
  responsabilidad de los metadatos.
- No hay filtro por serie ni cabeceras de grupo todavía.
- Tests cubren Calibre, EPUB3, `BookEntry` y el orden.
