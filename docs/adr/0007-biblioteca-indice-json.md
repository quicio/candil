---
id: adr-0007
status: aceptada
version: 1.1.0
updated: 2026-11-06
---

# ADR-0007 — La biblioteca indexa carpetas, no copia archivos

## Contexto

M2 introduce la biblioteca: ver los libros como estantería, ordenar, filtrar,
buscar y abrir. Hay que decidir dónde viven los libros y dónde el índice.

El usuario ya tiene los libros organizados en carpetas propias (Descargas,
proyectos, bibliotecas de Calibre). Un lector que copia todo a un directorio
propio duplica gigabytes y rompe esa organización.

Volumen esperado: cientos de libros. Escrituras: al importar y al cerrar un libro.

## Decisión

1. **Referencia en el lugar**: la biblioteca guarda rutas absolutas canónicas a
   los `.epub`. No los mueve ni los copia.
2. **Carpetas vigiladas**: el usuario agrega carpetas; se escanean al agregarlas y
   al pedir refrescar (no hay watcher del sistema en M2).
3. **Índice en `library.json`** junto a `config.json`, con una entrada por libro:

   ```json
   {
     "path": "/ruta/libro.epub",
     "title": "…", "author": "…", "lang": "es",
     "cover": "covers/ab12cd34.png",
     "size": 1234567, "mtime": 1762000000,
     "progress": { "chapter": 13, "offset": 366.9 },
     "added": 1762000000, "last_opened": 1762000001
   }
   ```

4. **Clave de identidad**: ruta canónica. Si el archivo desaparece, la entrada
   queda marcada `missing: true` en vez de borrarse (el usuario puede tener el
   disco desmontado).
5. **Reescaneo incremental**: se compara `mtime` + `size` para no reabrir el zip
   de libros que no cambiaron.

## Consecuencias

Buenas:

- Cero duplicación de disco; respeta la organización existente.
- Escaneo de 50 libros ≈ abrir 50 zips una vez; con caché, instantáneo al reabrir.
- El índice es texto: se puede inspeccionar, migrar y reparar a mano.

Malas:

- Rutas absolutas: mover un libro lo marca como faltante (no hay huella de
  contenido todavía; ver "revisar si").
- Sin indexado full-text: la búsqueda de M2 es sobre metadatos y nombre de archivo.
- Si el usuario borra un archivo, la entrada queda visible hasta que se limpie.

## Alternativas descartadas

- **Biblioteca administrada** (copiar a `~/Library/Application Support/lector/books`):
  ordena el caos, pero duplica archivos y obliga a "importar" en vez de "apuntar".
  Se puede ofrecer después como opción explícita, no como comportamiento por defecto.
- **SQLite + FTS5**: resuelve consultas y full-text, pero mete C, ~1.5 MB y
  migraciones, para un corpus de cientos. Revisar si se superan ~5.000 libros o si
  M4 pide búsqueda global real.
- **Watcher (`notify`)**: actualización automática, pero agrega dependencia,
  hilos y casos borde (redes montadas). Postergado.

## Revisar si

- El corpus supera 5.000 libros → SQLite.
- Se quiere detectar archivos movidos → agregar hash parcial (primeros 64 KB) a la
  identidad.

## Implementación (2026-11-06)

`src/library.rs` implementa T1–T3: rutas canónicas, `schema_version: 1`,
`library.json` atómico vía `src/store.rs`, escaneo recursivo sin symlinks e
incremental por `size` + `mtime`. Medido sobre 43 EPUB: 1.57 s la primera vez y
0.02 s con caché. `missing` se marca al reescanear una carpeta. La portada y la
pantalla quedan en ADR-0008 y spec 20.
