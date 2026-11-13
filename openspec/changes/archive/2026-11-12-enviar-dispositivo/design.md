## Context

La biblioteca tiene acciones de abrir, editar, mostrar en Finder y quitar. No
había forma de copiar un EPUB a un lector externo. Kobo expone una carpeta `.kobo`
en la raíz del volumen; Kindle no lee EPUB por USB.

## Goals / Non-Goals

**Goals:**

- Copiar un EPUB a un Kobo montado con un clic.
- Permitir cualquier dispositivo/carpeta montada.
- No pisar archivos ni tocar el original.

**Non-Goals:**

- Conversión de formatos, email a Kindle o API de Amazon.
- Borrar o mover archivos del dispositivo.
- Copia automática al conectar.

## Decisions

### 1. Detección de Kobo por `.kobo`

`kobo_in(root)` busca directorios con `.kobo` dentro. `kobo_dir()` revisa
`/Volumes` (macOS), `/media` y `/run/media` (Linux) y letras de unidad (Windows).
El diálogo muestra el primero detectado.

### 2. Destino y nombres únicos

En Kobo se prefiere `Books/` si existe; si no, la raíz. En una carpeta genérica se
usa la carpeta elegida. `unique_destination` agrega ` (2)`, ` (3)`, etc. antes de
la extensión si el nombre ya existe. Nunca se sobrescribe.

### 3. Copia explícita

`std::fs::copy` en el hilo de UI (los EPUB son pequeños) y toast con el destino.
Si falla, se muestra el error y el original queda intacto. No se mueve ni borra
nada.

### 4. Kindle

El diálogo incluye una nota: Kindle normalmente no lee EPUB por USB; usar Send to
Kindle o un archivo convertido. La carpeta genérica sirve para copiar un archivo
ya convertido, pero no se convierte.

## Risks / Trade-offs

- **Dispositivo lleno** → el error de copia se muestra; no se borra nada.
- **Kobo con nombre existente** → se crea una copia con sufijo; puede duplicar.
- **Detección por `.kobo`** → si el volumen no está montado o usa otra estructura,
  el usuario elige la carpeta manualmente.
- **Kindle** → no se promete compatibilidad.

## Migration Plan

Sin migración ni datos. Rollback: quitar la acción y el diálogo.
