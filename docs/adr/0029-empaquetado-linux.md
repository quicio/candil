# ADR-0029: Empaquetado para Linux (nfpm, tarball y AUR)

- **Estado:** propuesto
- **Fecha:** 2026-11-14
- **Área:** distribución, empaquetado, CI/CD, Linux

## Contexto

Hasta hoy Linux se instala bajando `candil-linux-x86_64` y dejándolo en el
`PATH`. Eso deja tres huecos:

1. Sin `.desktop`, iconos ni `MimeType`, el lector no aparece en el menú ni se
   puede abrir un `.epub` con doble clic: el usuario tiene que crear la entrada
   a mano.
2. Sin paquete, actualizar es volver a descargar y reemplazar el archivo; no hay
   `apt`/`dnf`/`pacman` que lleve la cuenta.
3. Sin metadatos AppStream, ni las tiendas de software ni `appstreamcli` pueden
   mostrar la app, y Flathub queda descartado de entrada.

El binario es un único ELF que enlaza solo `libc`, `libm` y `libgcc_s`; las libs
de ventana entran por `dlopen` (`libwayland-client`, `libxkbcommon`, `libX11`,
`libXcursor`, `libEGL`/`libGL`). No hay GTK ni Qt: los diálogos usan
`ashpd`/`xdg-desktop-portal` y el llavero, `zbus` contra el servicio de secretos.
Eso hace que empaquetar sea barato y que las dependencias reales no se vean en
`ldd`.

Las releases son automáticas y frecuentes (`auto-release` en cada push con
código), así que cualquier empaquetado que exija edición manual por versión
queda viejo en días.

## Decisión

### Un solo archivo de empaquetado: nfpm

`packaging/nfpm.yaml` describe contenido y dependencias una vez; `nfpm` genera
`.deb`, `.rpm` y `.pkg.tar.zst`. Es un binario de Go sin dependencias, así que
corre igual en el runner de Ubuntu y en la máquina de desarrollo. Se descartan
`cargo-deb` + `cargo-rpm` (dos herramientas, dos manifest) y `fpm` (Ruby).

Contenido instalado en los tres formatos:

| ruta | qué |
|---|---|
| `/usr/bin/candil` | binario |
| `/usr/share/applications/candil.desktop` | entrada de menú + `MimeType=application/epub+zip` |
| `/usr/share/icons/hicolor/{256x256,scalable}/apps/candil.{png,svg}` | iconos |
| `/usr/share/metainfo/org.quicio.candil.metainfo.xml` | AppStream |
| `/usr/share/man/man1/candil.1.gz` | man page |
| `/usr/share/licenses/candil/LICENSE` | licencia |

Sin maintainer scripts: los triggers de `desktop-file-utils` y
`gtk-update-icon-cache` de cada familia ya refrescan las cachés.

Dependencias declaradas (abren por `dlopen`, no salen en `ldd`):
`libxkbcommon`, `libwayland-client`, `libgl1`/`mesa-libGL`/`mesa`, y en
Debian/Fedora además `libX11` y `libXcursor` para el fallback X11. Recomendadas:
`xdg-desktop-portal` (+ backend gtk). Sugerida: `gnome-keyring`.

### Tarball portable con todo

`packaging/build.sh` arma además `candil-<ver>-linux-x86_64.tar.gz` con el mismo
contenido y un `INSTALL.txt` que lista los `install -D` para `~/.local`. Sirve
para quien no tiene root o no quiere un paquete, y es la fuente del paquete del
AUR. El `mtime` de los archivos es el del commit (`SOURCE_DATE_EPOCH`), así que
el tarball es reproducible y su checksum no cambia entre builds iguales.

El asset crudo `candil-linux-x86_64` se mantiene: `update.rs` lo busca por
nombre y la actualización asistida (ADR-0016) no se toca.

### `packaging/install.sh` mientras no hay AUR

El paquete del AUR depende de una cuenta, y el registro está cerrado por tandas.
Para que instalar no sea "bajar un archivo y acordarse de los permisos", el
tarball viene con un instalador: baja la última release, verifica el SHA-256
contra `SHA256SUMS.txt` y copia los siete archivos en `~/.local` (o
`/usr/local` con `--system`). Refresca `mimeinfo.cache` e `icon-theme.cache`,
así que la asociación `.epub` funciona sin pasos extra.

No es un repo de pacman propio: eso pediría clave PGP, hosting y una base de
datos firmada, y el AUR debería cubrirlo en semanas. Si el registro sigue
cerrado, ese es el paso siguiente, no el primero.

### AUR `candil-bin`

Arch es la plataforma de desarrollo, así que el AUR es la vía principal de
instalación con `paru`/`yay`. Se elige `-bin` (sin compilar) porque compilar
`eframe` en un `chroot` de AUR tarda minutos por cada release; el tarball ya
trae `.desktop`, iconos y man page.

`.github/workflows/aur.yml` corre con `release: published`, lee `pkgver` y el
`sha256sums` del tarball desde `SHA256SUMS.txt` y publica con
`KSXGitHub/github-actions-deploy-aur`. Nada de editar el `PKGBUILD` a mano.
Un `candil` (compilado) o `candil-git` quedan fuera por ahora: `conflicts`
declara el nombre y se pueden sumar después sin tocar este diseño.

## Alternativas descartadas

- **AppImage.** Herramientas envejecidas (`linuxdeploy` requiere FUSE) y aporta
  poco sobre el tarball, que ya es un archivo único sin instalación.
- **Flatpak/Flathub.** Es la mejor respuesta para alcance masivo, pero pide
  manifest, runtime, permisos de portal para la biblioteca EPUB y pasar la cola
  de revisión de Flathub; el `.metainfo.xml` de este ADR es el requisito previo,
  así que queda como paso siguiente y no como primer paso.
- **PPA / COPR / OBS.** Infraestructura por familia sin ganar nada frente a
  publicar el paquete como asset.
- **PKGBUILD que compila desde el tag.** Correcto para puristas, caro de
  mantener con releases automáticas; se puede agregar reusando `aur.yml`.

## Consecuencias

- Instalar deja de ser "bajar un archivo": `paru -S candil-bin`,
  `apt install ./candil_*.deb`, `dnf install ./candil-*.rpm` o
  `pacman -U`.
- La entrada de menú y la asociación de `.epub` funcionan sin pasos manuales.
- `packaging/build.sh` es la única puerta: CI y local producen lo mismo, y el
  script valida `.desktop`, AppStream y los tres paquetes antes de publicar.
- Costo nuevo: mantener `nfpm.yaml` cuando cambien dependencias o rutas, y
  regenerar `pkgver`/`sha256sums` (automático en `aur.yml`).
- Riesgo asumido: el paquete de Arch que emite nfpm dice `Unknown Packager` en
  `.PKGINFO` (nfpm no expone ese campo). Es cosmético; el paquete real de Arch
  es el del AUR, donde `makepkg` sí lo completa.
- La actualización asistida sigue igual: descarga y verifica, no reemplaza. Un
  usuario con paquete puede ignorarla y usar su gestor.
