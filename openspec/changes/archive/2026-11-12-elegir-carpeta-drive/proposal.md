## Why

El acceso directo a Google Drive agregaba `My Drive` completo sin dejar elegir una
subcarpeta, y no advertía qué pasa con los EPUB que están solo en la nube. El
usuario quiere indicar un directorio concreto y entender el costo de descarga.

## What Changes

- **Añadir Google Drive…** abre el selector de carpetas en la raíz de Google Drive
  para elegir una subcarpeta, en vez de indexar `My Drive` entero.
- Si la subcarpeta ya está en la biblioteca, avisa sin duplicar.
- El aviso al indexar explica que Drive descarga los EPUB al abrirlos y que
  conviene marcar la carpeta como disponible sin conexión.
- La raíz se sigue detectando en macOS (`~/Library/CloudStorage/GoogleDrive-*/My
  Drive`) y `~/Google Drive`.
- Sin OAuth ni API de Google.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: elegir subcarpeta de Google Drive y advertir sobre
  archivos solo en la nube.

## Scope

**Incluido:** picker desde la raíz de Drive, deduplicación, aviso de descarga,
docs y tests existentes de detección.

**No incluido:** OAuth, Drive API, descarga selectiva propia ni detección de
placeholders.

## Acceptance Criteria

- [x] **Añadir Google Drive…** permite elegir una subcarpeta.
- [x] No se agrega `My Drive` completo automáticamente.
- [x] Si la carpeta ya está indexada, no se duplica.
- [x] El mensaje aclara el comportamiento de archivos no descargados.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`; sin dependencias nuevas.
- Docs: ADR-0026, HANDOFF, ROADMAP y README.
