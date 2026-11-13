## Why

La búsqueda full-text reabre y parsea todos los EPUB en cada consulta. La primera
búsqueda es costosa; refinarla o buscar otra frase vuelve a pagar el mismo costo.
Un índice persistente resolvería la velocidad pero consume disco y hay que
invalidarlo.

## What Changes

- Caché **LRU en memoria** de `SearchIndex` por libro, con tope de **64 MB**.
- Clave `ruta + tamaño + mtime`: un EPUB editado no reutiliza texto viejo.
- `Arc<SearchIndex>` compartido en `Arc<Mutex<SearchCache>>` entre búsquedas.
- La primera búsqueda de una sesión construye; las siguientes reutilizan.
- No se persiste en disco ni entra en el respaldo.

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `library-experience`: caché en memoria de índices de búsqueda full-text.

## Scope

**Incluido:** caché LRU, clave por archivo, tope, evicción, tests, ADR-0028 y
documentación.

**No incluido:** índice persistente, cachear resultados por consulta, invalidación
manual o búsqueda en segundo plano permanente.

## Acceptance Criteria

- [x] La segunda búsqueda reutiliza índices ya construidos.
- [x] Un cambio de tamaño/mtime invalida la entrada.
- [x] El caché se limita a 64 MB y evicta las entradas menos usadas.
- [x] No se escribe ningún índice en disco.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/app.rs`; sin dependencias nuevas.
- Docs: ADR-0028, HANDOFF, ROADMAP y README.
