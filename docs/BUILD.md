# Compilar `candil` en macOS, Linux y Windows

La vía recomendada es **compilar nativo en cada sistema**. Es una app de escritorio
con OpenGL (egui/eframe), diálogos nativos (`rfd`) y TLS (`rustls`); el
cross-compiling funciona, pero suele necesitar toolchains y librerías del sistema
del destino. La matriz de GitHub Actions de `.github/workflows/build.yml` hace
builds nativos en los tres sistemas.

## Requisitos comunes

- Rust estable (`rustup`).
- `cargo` y `rustc` en el `PATH`.
- Para tests y build: `cargo test`, `cargo clippy --all-targets -- -D warnings`,
  `scripts/check.sh` (bash; en Windows usar Git Bash o WSL).

Perfiles:

```bash
cargo build --profile fast    # iterar, ~3–6 s por cambio
cargo build --profile release # uso diario, LTO thin
cargo build --profile dist    # distribuir, LTO fat, ~7.2 MB
```

El binario queda en `target/<perfil>/candil` (macOS/Linux) o
`target\<perfil>\candil.exe` (Windows). Con `dist`, en `target/dist/`.

## macOS

Requisitos:

```bash
xcode-select --install       # Command Line Tools
rustup default stable
```

Build:

```bash
cargo build --profile dist
./target/dist/candil
```

Binario universal (Apple Silicon + Intel):

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --profile dist --target aarch64-apple-darwin
cargo build --profile dist --target x86_64-apple-darwin
lipo -create \
  target/aarch64-apple-darwin/dist/candil \
  target/x86_64-apple-darwin/dist/candil \
  -output candil-universal
```

La app no está empaquetada como `.app`; para distribuirla como aplicación de
Finder hay que crear el bundle (`Contents/MacOS/candil`, `Info.plist`, icono).

### macOS: `Candil.app` con icono

El binario suelto funciona desde la terminal, pero Finder/Dock usan el icono
del bundle. Para armar `Candil.app`:

```bash
cargo build --profile dist
scripts/make_app.sh target/dist/candil "$HOME/Applications/Candil.app"
open "$HOME/Applications/Candil.app"
```

El script copia el binario, `assets/candil.icns` y un `Info.plist` con
`CFBundleIdentifier=com.quicio.candil`, firma ad-hoc y quita la cuarentena.
También se puede envolver un binario ya instalado:

```bash
scripts/make_app.sh "$HOME/.local/bin/candil" "$HOME/Applications/Candil.app"
```

El icono fuente es `assets/candil-icon.svg`; los PNG y el `.icns` se generan
desde ahí.

## Linux

Dependencias típicas en Ubuntu/Debian:

```bash
sudo apt update
sudo apt install -y \
  build-essential pkg-config \
  libx11-dev libxi-dev libxcursor-dev libxrandr-dev \
  libxkbcommon-dev libwayland-dev libgl1-mesa-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

Fedora:

```bash
sudo dnf install gcc gcc-c++ pkg-config libX11-devel libXi-devel \
  libXcursor-devel libXrandr-devel libxkbcommon-devel wayland-devel \
  mesa-libGL-devel libxcb-devel
```

Arch:

```bash
sudo pacman -S base-devel pkg-config libx11 libxi libxcursor libxrandr \
  libxkbcommon wayland mesa libxcb
```

Build:

```bash
cargo build --profile dist
./target/dist/candil
```

Notas:

- `rfd` usa **xdg-desktop-portal** para los diálogos. Si no abre el selector de
  archivos, instalá `xdg-desktop-portal` y el backend de tu escritorio
  (`xdg-desktop-portal-gtk`, `-kde`, etc.).
- Necesitás drivers OpenGL funcionando. Como último recurso:
  `LIBGL_ALWAYS_SOFTWARE=1 ./target/dist/candil`.
- Funciona con X11 y Wayland; si el backend falla, probá desde una sesión X11.
- El binario no incluye un `.desktop` ni un AppImage; se puede agregar después.

## Windows

Requisitos:

1. Instalar Rust con el toolchain **MSVC**:
   `rustup default stable-msvc`.
2. Instalar **Visual Studio Build Tools** con la carga de trabajo
   **Desktop development with C++** (linker `link.exe`, Windows SDK). No hace
   falta Visual Studio completo.

Build (PowerShell o CMD):

```powershell
cargo build --profile dist
.\target\dist\candil.exe
```

La app usa diálogos nativos de Windows y no necesita webview. El binario de
release se abre con una consola porque el proyecto conserva los flags de
diagnóstico (`--inspect`, `--scan`) que imprimen por stdout; si querés ocultarla,
se puede agregar `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
en `src/main.rs`, a costa de perder esa salida en release.

Variables de entorno en Windows:

```powershell
$env:OPENCODE_API_KEY = "tu-clave"     # solo esta sesión
setx OPENCODE_API_KEY "tu-clave"       # persistente, nuevas sesiones
```

## Clave de IA y lanzadores

`OPENCODE_API_KEY` se lee del entorno. Las apps abiertas desde Finder, el menú de
Linux o el acceso directo de Windows **no heredan** el entorno del shell. Por
ahora:

- ejecutá desde una terminal (`export OPENCODE_API_KEY=…` o `$env:…`), o
- definí la variable en el entorno del usuario y reiniciá el lanzador.

La integración con Keychain/libsecret queda para una etapa posterior.

## Cross-compiling

- **macOS → macOS universal:** soportado con `rustup target add` + `lipo`
  (ver arriba).
- **macOS/Linux → Windows:** posible con `cargo-zigbuild` o una VM Windows, pero
  `ring` (TLS) y `eframe` pueden requerir toolchains adicionales. Para entregar,
  es más confiable compilar en Windows.
- **macOS → Linux:** no se recomienda (glibc, X11/Wayland, GL). Usá una VM Linux,
  Docker con `cross` o el job de CI.
- **Tests:** no requieren red; el test live de Wiktionary/MyMemory está marcado
  `#[ignore]` y se corre a mano:
  `cargo test --lib red_real_wiktionary_y_mymemory -- --ignored`.

## CI

`.github/workflows/build.yml` compila y testea en `ubuntu-latest`,
`macos-latest` y `windows-latest`, y sube el binario `dist` como artefacto.
No configura claves ni corre el test live. Para builds de release, etiquetá un
tag o descargá el artefacto del job.

## Dónde quedan los datos

| sistema | directorio |
|---|---|
| macOS | `~/Library/Application Support/candil/` |
| Linux | `~/.config/candil/` (o `$XDG_CONFIG_HOME/candil/`) |
| Windows | `%APPDATA%\candil\` |

Ahí viven `config.json`, `library.json`, `annotations.json`, `consultas.json` y
`covers/`.
