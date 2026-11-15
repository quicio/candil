# ADR-0030: Instalar actualizaciones desde la UI (Linux)

- **Estado:** aceptado
- **Fecha:** 2026-11-14
- **Área:** distribución, configuración, seguridad

## Contexto

ADR-0016 dejó la actualización en modo asistido: Candil consulta GitHub a pedido,
verifica SHA-256 y deja el asset en Descargas, pero nunca reemplaza el binario.
Eso obliga a mover archivos a mano. En Linux, cuando Candil está instalado en
`~/.local/bin` o `~/bin`, el propio usuario es dueño del archivo y puede
reemplazarlo sin root. Además, `rename(2)` sobre un ejecutable en ejecución es
atómico: el proceso actual sigue usando el inodo viejo hasta reiniciarse.

En macOS el binario vive dentro de un `.app` firmado ad-hoc; reemplazarlo exige
volver a firmar el bundle. En Windows no se puede sobrescribir un `.exe` en
ejecución. Las instalaciones por gestor de paquetes (`/usr/bin`, `/nix/store`,
Flatpak, AppImage) tampoco deben auto-reemplazarse: el gestor quedaría
desincronizado.

## Decisión

1. **Solo Linux y solo instalación de usuario.** La opción aparece únicamente
   si el asset existe y `SELF_INSTALL_SUPPORTED` es verdadero. Al instalar se
   canonicaliza `current_exe`; si no está bajo `$HOME` o contiene un componente
   `target`, se rechaza con un mensaje que indica usar el gestor de paquetes o
   compilar desde el repo. Un binario en `/usr`, `/opt`, `/nix/store`, Flatpak,
   AppImage o `target/` nunca se toca.
2. **Doble verificación.** La descarga ya verificó el SHA-256 contra
   `SHA256SUMS.txt`; antes de escribir se vuelve a leer el archivo, se comprueba
   el mismo hash y se exige que empiece con el magic `\x7fELF`. El archivo
   descargado nunca se ejecuta.
3. **Backup y reemplazo atómico.** Se copia el binario actual a `candil.bak` en
   el mismo directorio. La actualización se escribe en un temporal
   `.candil.update-*.tmp` del mismo directorio, con permisos `0o755` y
   `fsync`; después se hace `rename` sobre el binario actual. Si algo falla, se
   borra el temporal y el original queda intacto.
4. **Estados explícitos.** Descargado → **Instalar actualización** → Instalando →
   Instalado → **Reiniciar ahora**. La ruta de instalación se guarda antes del
   `rename`: después del reemplazo `current_exe` devuelve `… (deleted)` para el
   inodo viejo y no sirve para relanzar. El reinicio guarda los datos, lanza esa
   ruta con los mismos argumentos y cierra sin volver a guardar. Si el lanzamiento
   falla, la app sigue abierta y muestra el error; el usuario puede reiniciar a
   mano.
5. **Sin chequeo automático.** La consulta sigue siendo una acción explícita y
   no se ejecuta al iniciar. La única petición nueva es la misma descarga de
   ADR-0016.
6. **macOS y Windows quedan asistidos.** Siguen mostrando el archivo descargado y
   las acciones de revelarlo/abrir el release, sin tocar el `.app` ni el `.exe`
   en ejecución.

## Alternativas descartadas

- **Script de actualización desde el repo:** útil para desarrollo, pero no es
  una actualización desde la interfaz y no sirve para usuarios sin toolchain.
- **Ejecutar el archivo descargado:** permitiría delegar el reemplazo a un
  instalador, pero ejecuta un binario recién bajado y amplía la superficie de
  ataque. Se descarta.
- **Reemplazar directo en Descargas:** evita el temporal, pero pierde atomicidad
  y puede dejar el binario a medias si falla el disco.
- **Autoactualizar también macOS/Windows:** multiplica el riesgo por plataforma
  (firma ad-hoc, archivo bloqueado). Queda para otra decisión.
- **Chequeo al iniciar:** contradice la regla de que nada sale a la red sin una
  acción explícita.

## Consecuencias

- Los usuarios de Linux con instalación por usuario actualizan desde
  **Configuración → Avanzado** sin salir de la app.
- Queda un `candil.bak` junto al binario y el asset original en Descargas.
- Las instalaciones por paquete, Flatpak, AppImage o Nix se rechazan con un
  mensaje claro.
- ADR-0016 sigue vigente para macOS, Windows y todos los casos no autoinstalables.
- Tests: descarga + hash, rechazo de archivo no-ELF, backup y reemplazo atómico
  en un directorio temporal.
