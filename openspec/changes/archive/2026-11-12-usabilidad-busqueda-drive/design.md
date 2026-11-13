## Context

La búsqueda full-text se lanzaba con Enter y el panel usaba `PANEL_MD` (420 px).
Google Drive no tenía ningún camino de entrada. El modelo de biblioteca ya indexa
carpetas locales sin copiar archivos (ADR-0007).

## Goals / Non-Goals

**Goals:**

- Hacer visible el modo de búsqueda en contenido.
- Dar espacio a los resultados.
- Permitir Google Drive sin introducir credenciales ni otra cuenta.

**Non-Goals:**

- OAuth/Drive API, descarga a `covers/` o sincronización propia.
- Indexar automáticamente todas las carpetas de nube.

## Decisions

### 1. Toggle Contenido

La barra tiene un checkbox etiquetado **Contenido** (o **Texto** en compacto).
Con el toggle apagado, Enter no lanza la búsqueda full-text. Al activarlo:

- si hay una consulta de 3+ caracteres, inicia la búsqueda;
- si no, muestra el mínimo de caracteres.
El campo de búsqueda sigue filtrando metadatos en cada tecla.

### 2. Panel relativo a la ventana

`window_w = (screen.width * 0.72).clamp(520, 920)` y
`window_h = (screen.height * 0.72).clamp(420, 760)`. El scroll interno descuenta
el encabezado.

### 3. Google Drive local

`google_drive_candidates(home)` devuelve:

- macOS: `~/Library/CloudStorage/GoogleDrive-*/My Drive`.
- además de `~/Google Drive`.

`google_drive_dir()` elige el primero que exista. **Añadir Google Drive…** aparece
en el menú Carpetas y en el estado vacío si se detecta; indexa con
`start_scan(Some(dir))`. Si no hay carpeta, el toast explica Drive para escritorio
o **Añadir carpeta…**.

**Por qué no OAuth:** requiere proyecto de Google Cloud, client ID, redirect,
tokens, caché y un ADR de privacidad; además duplica archivos. Se pospone.

## Risks / Trade-offs

- **Drive no instalado** → no hay botón en el estado vacío; el menú explica.
- **Carpeta sincronizada parcialmente** → el escaneo normal marca faltantes; no se
  inventa contenido.
- **Panel más grande** → puede tapar la grilla, pero es una ventana temporal.
- **Toggle apagado** → Enter ya no busca contenido; el toggle es la señal
  explícita.

## Migration Plan

Sin migración ni datos. Rollback: quitar el toggle y el menú; la búsqueda vuelve a
Enter.
