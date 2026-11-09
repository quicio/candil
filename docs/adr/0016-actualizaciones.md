# ADR-0016: Actualización asistida desde la aplicación

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** distribución, configuración, red

## Contexto

Candil se distribuye por GitHub Releases con tres binarios y
`SHA256SUMS.txt`. La app no comprueba si hay una versión nueva ni ayuda a
descargarla; el usuario tiene que recordar la página, elegir el asset correcto y
verificar el checksum a mano.

Un autoactualizador completo es riesgoso: en macOS el `.app` está firmado
ad-hoc y reemplazar el ejecutable exige re-firmar; en Windows no se puede
sobrescribir un `.exe` en ejecución. Además, la regla del proyecto es que nada
sale a la red sin una acción explícita.

## Decisión

Implementar una actualización **asistida**:

1. **Consulta manual.** Configuración → Avanzado → “Buscar actualizaciones”
   llama a `GET https://api.github.com/repos/quicio/candil/releases/latest` en
   un hilo. No hay chequeo al iniciar ni en segundo plano.
2. **Comparación numérica.** Se normaliza `v0.5.0` y se comparan hasta tres
   componentes; si la remota no es mayor, se muestra “Estás al día”.
3. **Selección por plataforma.** macOS arm64 → `candil-macos-arm64`; Linux
   x86_64 → `candil-linux-x86_64`; Windows x86_64 →
   `candil-windows-x86_64.exe`. Otra combinación no tiene asset y se ofrece la
   página del release.
4. **Descarga verificada.** Se bajan el asset y `SHA256SUMS.txt`; se busca la
   línea del asset, se calcula SHA-256 con `sha2` y se compara. Si no coincide,
   no se escribe nada.
5. **Destino y permisos.** El archivo se guarda en `dirs::download_dir()` (o
   `~/Downloads` / `data_dir/updates` como fallback) con un temporal + rename y
   permisos `0o755` en Unix. Nombre:
   `candil-<versión>-<asset>`.
6. **Sin autoinstalación.** La app no reemplaza el binario ni el `.app`, no
   reinicia y no ejecuta el archivo. Ofrece “Mostrar en Finder/Explorer” y
   “Abrir release”.
7. **Privacidad.** La única petición es a `api.github.com` y a los assets del
   release cuando el usuario pulsa el botón. No se envían libros, rutas,
   anotaciones, historial ni la clave de IA.

`sha2` pasa a dependencia directa; ya estaba en el árbol vía
`zip`/`lzma-rust2`, así que no se agrega un crate nuevo. `ureq`, `serde_json`,
`serde` y `dirs` ya eran dependencias.

## Alternativas descartadas

- **Autoactualización completa:** descarga, reemplazo, backup, reinicio y
  re-firma. Es lo que más comodidad daría, pero multiplica los riesgos por
  plataforma y necesita un diseño y pruebas aparte.
- **Solo abrir la página del release:** mínimo, pero no resuelve elegir el
  binario ni verificar el checksum, que es la fricción real.
- **Verificar con `shasum`/`sha256sum`/`certutil`:** evita `sha2`, pero depende
  de herramientas externas, tiene formatos distintos y es más frágil.
- **Chequeo automático al iniciar:** cómodo, pero contradice la regla de que
  nada sale a la red sin una acción explícita y agrega una petición no pedida.
- **Endpoint propio:** permitiría métricas y versiones, pero agrega backend e
  infraestructura para un proyecto local-first.

## Consecuencias

- La instalación sigue siendo manual: el usuario mueve el binario o reconstruye
  `Candil.app`. La app lo dice explícitamente.
- GitHub limita a 60 consultas/hora sin autenticar; una consulta manual por clic
  es suficiente. Si se alcanza, se muestra el 403 y se puede abrir el release.
- El `dist` pasa de **7.250.976 B** a **7.284.272 B** (+33.296 B); sigue por
  debajo de 8 MB.
- Sin telemetría ni datos del usuario; el texto de la UI lo explica.
- Tests: comparación de versiones, parseo del JSON, selección de asset,
  checksums, SHA-256 y una descarga completa contra un servidor local.
- Quedan fuera: autoactualización, canales beta, chequeo opt-in al iniciar y
  notarización.
