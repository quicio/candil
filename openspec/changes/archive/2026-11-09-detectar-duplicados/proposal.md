## Why

Una biblioteca que indexa carpetas puede acumular la misma obra dos veces: una
copia descargada, otra con otro nombre de archivo o una edición repetida. Hoy no
hay forma de verlo sin comparar tarjeta por tarjeta. La detección debe ser
conservadora y no borrar nada por su cuenta.

## What Changes

- La biblioteca calcula posibles duplicados por **título y autor normalizados**
  (mayúsculas y acentos incluidos).
- El filtro suma **Duplicados**, que muestra solo los libros que comparten clave
  con al menos otro.
- Las tarjetas duplicadas llevan una insignia **Posible duplicado ×N** y un
  tooltip que explica el criterio.
- No se borra ni se mueve ningún archivo: quitar una entrada sigue siendo una
  acción explícita del menú contextual.
- Banderas `--filter duplicates` para capturas y verificación.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: detección conservadora de posibles duplicados y filtro
  para revisarlos.

## Scope

**Incluido:** clave título+autor normalizada, conteo, filtro, insignia, tooltip,
bandera de desarrollo y tests.

**No incluido:** hash de contenido, comparación byte a byte, borrado automático,
fusión de entradas, elección de la “copia buena”, edición de metadatos.

## Acceptance Criteria

- [x] Dos libros con el mismo título y autor, ignorando mayúsculas y acentos,
      aparecen como posibles duplicados.
- [x] Un libro único no aparece en el filtro **Duplicados**.
- [x] La tarjeta muestra la insignia con la cantidad de copias.
- [x] El tooltip aclara que el criterio es título y autor.
- [x] Candil no borra ni mueve archivos automáticamente.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`, `src/main.rs`.
- Sin cambios en `library.json` ni dependencias; la detección se calcula en
  memoria a partir de los metadatos ya indexados.
- Docs: HANDOFF, ROADMAP, README y spec de biblioteca.
