## Context

Candil se publica en GitHub Releases (`quicio/candil`) con tres assets
(`candil-macos-arm64`, `candil-linux-x86_64`, `candil-windows-x86_64.exe`) y
`SHA256SUMS.txt`. La app ya tiene `ureq` (con rustls), `serde_json`, `dirs` y
`sha2` en el árbol (este último vía `zip`/`lzma-rust2`). Configuración → Avanzado
ya muestra la versión y los archivos de datos.

La regla del proyecto es que nada sale a la red sin una acción explícita. La
actualización se dispara solo con un botón y no toca la instalación.

## Goals / Non-Goals

**Goals:**

- Comprobar la última release estable sin bloquear la UI.
- Mostrar versión, fecha y novedades.
- Descargar el asset correcto y verificar su SHA-256 antes de guardarlo.
- Dejar el archivo en Descargas y ofrecer mostrarlo/abrir el release.
- Mantener la app y el binario actuales intactos.

**Non-Goals:**

- Autoactualización, reemplazo del binario/.app, reinicio, firma o notarización.
- Canales beta o releases preliminares.
- Chequeo automático al iniciar.
- Telemetría o envío de datos del usuario.
- Descargar sin una acción explícita.

## Decisions

### 1. Módulo `update.rs` separado

`update.rs` concentra la red, el parseo del JSON de GitHub, la comparación de
versiones, la selección de asset y la verificación SHA-256. La UI solo maneja
estados. **Alternativa descartada:** meter todo en `app.rs`; mezcla red, parseo y
UI y hace imposible testear sin egui.

### 2. API de GitHub Releases

`GET https://api.github.com/repos/quicio/candil/releases/latest` con
`User-Agent: candil/<versión>` y `Accept: application/vnd.github+json`. Timeout
global de 20 s. Se ignoran `draft` y `prerelease`. **Alternativa descartada:**
parsear la página HTML del release; frágil y más pesado. **Alternativa
descartada:** un endpoint propio; no hay backend y agrega infraestructura.

### 3. Comparación semver numérica

Se normaliza `v0.5.0` → `0.5.0` y se comparan hasta tres componentes numéricos,
rellenando con ceros. Las releases son estables; un sufijo de pre-release no se
considera más nuevo que la misma versión estable. **Alternativa descartada:**
traer una librería semver; no hace falta para tags simples y suma dependencia.

### 4. Asset por plataforma

- macOS `aarch64` → `candil-macos-arm64`.
- Linux `x86_64` → `candil-linux-x86_64`.
- Windows `x86_64` → `candil-windows-x86_64.exe`.
- Otra combinación → sin asset; la UI ofrece abrir el release.

El asset se elige por nombre exacto; `SHA256SUMS.txt` se busca por nombre en la
misma release. **Alternativa descartada:** elegir por extensión o por tamaño;
ambiguo y propenso a bajar el binario equivocado.

### 5. Descarga y verificación

La descarga corre en un hilo. Se bajan los bytes del asset y el texto de
`SHA256SUMS.txt`; se busca la línea del asset, se calcula SHA-256 con `sha2` y se
compara en minúsculas. Solo si coincide se escribe el archivo final con
temporal + rename y permisos `0o755` en Unix. El nombre es
`candil-<versión>-<asset>`. Si algo falla, se borra el temporal.

`sha2` ya está en `Cargo.lock` por `zip`/`lzma-rust2`; pasa a dependencia directa
sin agregar un crate nuevo. **Alternativa descartada:** invocar `shasum`/
`sha256sum`/`certutil`; distinto por plataforma, parseo frágil y dependencia de
herramientas externas.

### 6. Destino: Descargas, no instalación

Se usa `dirs::download_dir()`; si no existe, `~/Downloads`; si tampoco,
`data_dir/updates`. La app no reemplaza el binario en ejecución ni el `.app`, no
ejecuta el archivo y no reinicia. **Por qué:** en macOS el bundle está firmado
ad-hoc y en Windows no se puede sobrescribir un `.exe` en ejecución; un
auto-reemplazo exige un diseño y pruebas por plataforma. **Alternativa
descartada:** autoactualizar ahora; riesgo alto y fuera del alcance elegido.

### 7. Estados y concurrencia

`UpdateState` es un enum (`Idle`, `Checking`, `UpToDate`, `Available`,
`Downloading`, `Downloaded`, `Error`). Un `mpsc::Receiver<UpdateMsg>` recibe el
resultado del hilo; `poll_update` lo consume en cada frame y pide repintado
mientras hay una operación. Un solo trabajo a la vez: el botón se deshabilita
mientras `update_rx` existe.

### 8. Privacidad

Ninguna petición al iniciar ni durante la lectura. La única salida es
`api.github.com` y los assets del release cuando el usuario pulsa el botón. No se
envían rutas, libros, anotaciones, historial ni la clave de IA. El texto de la UI
lo explica.

### 9. Abrir y mostrar

Se agregan helpers multiplataforma: `open_external(url)` (`open`, `xdg-open`,
`cmd /C start`) y `reveal_path(path)` (`open -R`, `xdg-open` al directorio,
`explorer /select,`). Son acciones explícitas y no descargan nada.

## Risks / Trade-offs

- **Límite de la API de GitHub** (60 consultas/hora sin autenticar) → una consulta
  manual por clic; si se alcanza, se muestra el 403 y se puede abrir el release a
  mano.
- **Release sin `SHA256SUMS.txt`** → no se permite descargar; se informa y se
  ofrece abrir la release.
- **Conexión lenta o asset grande** (7–12 MB) → hilo separado y timeout de 20 s;
  la UI sigue usable. Si falla, se reintenta.
- **Disco lleno** → no se escribe el archivo final; se borra el temporal y se
  informa.
- **Plataforma sin asset** (macOS Intel, ARM Linux) → se muestra el aviso y la
  página del release.
- **Expectativa de autoinstalación** → el botón dice “Descargar”, no “Actualizar”;
  el texto aclara que la instalación es manual.
- **Sin nueva dependencia** → `sha2` ya estaba en el árbol; el `dist` no debería
  crecer de forma significativa.

## Migration Plan

Sin migración de datos. La primera comprobación crea el bloque de actualizaciones
en Configuración → Avanzado; no se guarda nada en `config.json`. Rollback: quitar
el botón y el módulo; no hay estado persistido.

## Open Questions

Ninguna que cambie el alcance. Autoactualización, canales beta y chequeo opt-in
al iniciar quedan para una decisión posterior.
