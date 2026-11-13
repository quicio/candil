## Context

`add_google_drive_folder` tomaba `google_drive_dir()` y llamaba a `start_scan`
directamente, sin picker. Eso indexaba todo `My Drive` y no permitía elegir una
carpeta de estudio. Tampoco había un aviso sobre los archivos que Drive mantiene
solo en la nube.

## Goals / Non-Goals

**Goals:**

- Elegir una subcarpeta concreta dentro de Drive.
- Evitar duplicados.
- Explicar el costo de descarga sin prometer acceso a archivos no descargados.

**Non-Goals:**

- OAuth, API de Google o descarga selectiva.
- Detectar placeholders de File Provider.

## Decisions

### 1. Picker con raíz de Drive

`rfd::FileDialog::new().set_directory(root).pick_folder()` abre el selector en
la raíz detectada. El usuario elige `My Drive/Libros`, `My Drive/Novelas`, etc.
No se indexa la raíz completa por accidente.

### 2. Aviso de nube

Después de elegir, el toast dice que Drive descarga los EPUB al abrirlos y que
conviene marcar la carpeta como disponible sin conexión. No se intenta detectar
el estado de cada archivo; la app no puede garantizar acceso offline si Drive no
lo tiene.

### 3. Deduplicación

La ruta elegida se compara con `library.folders`; si ya existe, no se vuelve a
escanear.

## Risks / Trade-offs

- **Drive sin archivos descargados** → el escaneo puede disparar descargas y
  consumir disco. El aviso lo hace explícito; la solución completa es OAuth/cache.
- **Ruta elegida fuera de Drive** → el picker permite navegar; se indexa como
  carpeta normal.
- **Sin Drive instalado** → se mantiene el mensaje de fallback.

## Migration Plan

Sin migración. Rollback: volver a indexar `google_drive_dir()` directamente.
