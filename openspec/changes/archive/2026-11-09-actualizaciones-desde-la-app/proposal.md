## Why

Candil se distribuye por GitHub Releases, pero la app no avisa cuando hay una versión nueva ni ayuda a descargarla. El usuario tiene que recordar la página, elegir el binario correcto y verificar el checksum a mano.

Un updater asistido, explícito y sin reemplazos automáticos, cierra esa fricción sin poner en riesgo la instalación.

## What Changes

- **Configuración → Avanzado** muestra la versión actual y un botón **Buscar actualizaciones**.
- La consulta a la API pública de GitHub corre en un hilo y solo se dispara con una acción explícita.
- Si hay una versión nueva, se muestran el número, la fecha y un resumen de las novedades, con **Descargar** y **Abrir release**.
- **Descargar** baja el asset de la plataforma actual y `SHA256SUMS.txt`, verifica el SHA-256 y guarda el archivo en Descargas con permisos de ejecución en Unix.
- La app **no reemplaza** el binario ni el `.app`: la instalación sigue siendo manual y explícita.
- Si no hay binario para la plataforma, se informa y se ofrece la página del release.
- Tests de comparación de versiones, parseo del JSON, selección de asset y checksum; captura con un estado simulado.

## Capabilities

### New Capabilities

- `app-updates`: comprobación manual de releases, selección del binario por plataforma, descarga verificada y estados de la UI.

### Modified Capabilities

- Ninguna: no cambia el lector, la biblioteca ni las anotaciones.

## Scope

**Incluido:** consulta manual, comparación semver, notas del release, descarga del asset correcto, verificación SHA-256, guardado en Descargas, abrir el release y mostrar el archivo.

**No incluido:** autoactualización, reemplazo del binario/.app, reinicio automático, firma/notarización, canales beta, chequeo al iniciar, telemetría, descarga en segundo plano sin acción del usuario.

## Acceptance Criteria

- [x] **Buscar actualizaciones** consulta GitHub solo al hacer clic y no bloquea la UI.
- [x] Si la versión publicada es mayor, se muestran versión, fecha y novedades; si es igual o menor, “Estás al día”.
- [x] **Descargar** elige el asset correcto para macOS arm64, Linux x86_64 o Windows x86_64, verifica el SHA-256 y guarda el archivo en Descargas.
- [x] Si el checksum no coincide, no se guarda el archivo y se muestra un error.
- [x] Si no hay asset para la plataforma, se informa y se ofrece abrir el release.
- [x] **Abrir release** y **Mostrar en Finder/Explorer** funcionan en las tres plataformas.
- [x] No se envía ningún dato del libro ni de la biblioteca; la única petición es a `api.github.com` y a los assets del release.
- [x] La app no reemplaza ni ejecuta el archivo descargado.
- [x] `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `scripts/check.sh` pasan; el `dist` sigue por debajo de 8 MB.

## Impact

- `src/update.rs` (nuevo), `src/lib.rs`, `src/app.rs`, `src/main.rs`.
- `Cargo.toml`: `sha2` pasa a dependencia directa (ya está en el árbol vía `zip`/`lzma-rust2`; sin crate nuevo).
- `docs/adr/0016-actualizaciones.md`, `README.md`, `docs/HANDOFF.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md` y `AGENTS.md`.
- UI de Configuración → Avanzado; captura con `--update-demo`.
