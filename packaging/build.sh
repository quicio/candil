#!/usr/bin/env bash
# Empaqueta Candil para Linux: tarball portable + .deb + .rpm + paquete de Arch.
#
#   ./packaging/build.sh [directorio-de-salida]     # por defecto, out/
#
# Requiere el binario ya compilado en target/dist/candil (`cargo build
# --profile dist`) y nfpm en el PATH (https://nfpm.goreleaser.com).
#
# La versión sale de Cargo.toml; se puede forzar con VERSION=x.y.z.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
out="${1:-out}"
version="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
bin="target/dist/candil"
stage=".packaging"

[ -x "$bin" ] || { echo "falta $bin: corré 'cargo build --profile dist' primero" >&2; exit 1; }
[ -n "$version" ] || { echo "no pude leer la versión de Cargo.toml" >&2; exit 1; }

nfpm=$(command -v nfpm || true)
[ -n "$nfpm" ] || { echo "falta nfpm en el PATH (https://nfpm.goreleaser.com)" >&2; exit 1; }

echo "Candil $version → $out/"
rm -rf "$out" "$stage"
mkdir -p "$out" "$stage"

# Las páginas de manual van comprimidas en Debian, Fedora y Arch; nfpm no
# comprime solo, así que se prepara acá.
gzip -9nc packaging/candil.1 > "$stage/candil.1.gz"

echo "· paquetes nativos"
VERSION="$version" "$nfpm" package -f packaging/nfpm.yaml -p deb       -t "$out/" >/dev/null
VERSION="$version" "$nfpm" package -f packaging/nfpm.yaml -p rpm       -t "$out/" >/dev/null
# El campo "Encargado" de Arch sale como "Unknown Packager": nfpm no expone ese
# campo. Es cosmético (pacman no lo valida); para Arch de verdad está el AUR.
VERSION="$version" "$nfpm" package -f packaging/nfpm.yaml -p archlinux -t "$out/" >/dev/null

echo "· tarball portable"
tb="candil-$version-linux-x86_64"
mkdir -p "$stage/$tb"
install -m 0755 "$bin" "$stage/$tb/candil"
install -m 0644 packaging/candil.desktop "$stage/$tb/candil.desktop"
install -m 0644 packaging/org.quicio.candil.metainfo.xml "$stage/$tb/org.quicio.candil.metainfo.xml"
install -m 0644 "$stage/candil.1.gz" "$stage/$tb/candil.1.gz"
install -m 0644 LICENSE "$stage/$tb/LICENSE"
install -D -m 0644 assets/candil-icon-256.png "$stage/$tb/icons/256x256/apps/candil.png"
install -D -m 0644 assets/candil-icon.svg "$stage/$tb/icons/scalable/apps/candil.svg"
cat > "$stage/$tb/INSTALL.txt" <<EOF
Candil $version (linux x86_64)

Instalación por usuario, sin root:

  install -Dm755 candil              ~/.local/bin/candil
  install -Dm644 candil.desktop      ~/.local/share/applications/candil.desktop
  install -Dm644 icons/256x256/apps/candil.png ~/.local/share/icons/hicolor/256x256/apps/candil.png
  install -Dm644 icons/scalable/apps/candil.svg ~/.local/share/icons/hicolor/scalable/apps/candil.svg
  install -Dm644 org.quicio.candil.metainfo.xml ~/.local/share/metainfo/org.quicio.candil.metainfo.xml
  install -Dm644 candil.1.gz         ~/.local/share/man/man1/candil.1.gz

Dependencias en tiempo de ejecución: libwayland-client y libxkbcommon (Wayland)
o libX11 y libXcursor (X11), más un libEGL/libGL del driver. Los diálogos de
archivo usan xdg-desktop-portal. El llavero necesita gnome-keyring o KWallet;
sin servicio de secretos, exportá CANDIL_NO_KEYRING=1.
EOF
# mtime = fecha del commit: el tarball sale reproducible, así el sha256 que usa
# el PKGBUILD del AUR no cambia entre builds del mismo contenido.
export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct 2>/dev/null || date -u +%s)}"
tar -C "$stage" --sort=name --owner=0 --group=0 --numeric-owner \
    --mtime="@$SOURCE_DATE_EPOCH" \
    -czf "$out/$tb.tar.gz" "$tb"
rm -rf "$stage/$tb"

echo "· verificación"
desktop-file-validate packaging/candil.desktop
appstreamcli validate --no-net packaging/org.quicio.candil.metainfo.xml >/dev/null 2>&1 \
  && echo "  metainfo AppStream OK" || { echo "  metainfo AppStream INVÁLIDO" >&2; exit 1; }
command -v dpkg-deb >/dev/null && dpkg-deb -I "$out/candil_${version}-1_amd64.deb" >/dev/null && echo "  .deb OK"
command -v pacman   >/dev/null && pacman -Qip "$out/candil-${version}-1-x86_64.pkg.tar.zst" >/dev/null && echo "  .pkg.tar.zst OK"
command -v rpm      >/dev/null && rpm -qpl "$out/candil-${version}-1.x86_64.rpm" >/dev/null && echo "  .rpm OK"
tar -tzf "$out/$tb.tar.gz" >/dev/null && echo "  tarball OK"

echo
(cd "$out" && sha256sum ./* | sed 's|\./||')
