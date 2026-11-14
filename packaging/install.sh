#!/usr/bin/env bash
# Instalador de Candil para Linux.
#
#   curl -fsSLO https://raw.githubusercontent.com/quicio/candil/main/packaging/install.sh
#   less install.sh          # miralo antes de correrlo
#   bash install.sh          # por usuario, en ~/.local (sin root)
#   sudo bash install.sh     # para todos, en /usr/local
#
# Descarga el tarball de la última release, verifica su SHA-256 contra
# SHA256SUMS.txt y copia binario, .desktop, iconos, man page y metainfo.
#
# Opciones:
#   --system            instalar en /usr/local (requiere root)
#   --version X.Y.Z     instalar una versión puntual en vez de la última
#   --archive RUTA      usar un tarball local (para probar el instalador)
#   --dry-run           mostrar qué haría, sin escribir nada
#   -h, --help          esta ayuda
set -euo pipefail

REPO="quicio/candil"
ASSET_PREFIX="candil"

mode="user"
version=""
archive=""
dry_run=""

while [ $# -gt 0 ]; do
  case "$1" in
    --system) mode="system" ;;
    --version) version="${2:?falta la versión}"; shift ;;
    --archive) archive="${2:?falta la ruta}"; shift ;;
    --dry-run) dry_run=1 ;;
    -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "opción desconocida: $1" >&2; exit 2 ;;
  esac
  shift
done

if [ "$mode" = "system" ]; then
  prefix="/usr/local"
  if [ "$(id -u)" -ne 0 ]; then
    echo "para --system hay que correr con sudo (o usar el modo por usuario)" >&2
    exit 1
  fi
else
  prefix="${HOME:?HOME no está definida}/.local"
fi

say() { printf '· %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

fetch() { # fetch URL DESTINO
  if command -v curl >/dev/null; then
    curl -fsSL --retry 2 -o "$2" "$1"
  elif command -v wget >/dev/null; then
    wget -qO "$2" "$1"
  else
    die "hace falta curl o wget"
  fi
}

# --- resolver versión y tarball -------------------------------------------
if [ -n "$archive" ]; then
  [ -f "$archive" ] || die "no existe $archive"
  tarball="$archive"
  say "tarball local: $archive"
  version="${version:-$(basename "$archive" | sed -E "s/^${ASSET_PREFIX}-(.*)-linux-x86_64\.tar\.gz$/\1/")}"
else
  if [ -z "$version" ]; then
    api="https://api.github.com/repos/$REPO/releases/latest"
    version="$(fetch "$api" "$work/latest.json" && sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p' "$work/latest.json" | head -1)"
    [ -n "$version" ] || die "no pude leer la última versión de $api"
  fi
  base="${CANDIL_BASE_URL:-https://github.com/$REPO/releases/download/v$version}"
  tarball="$work/${ASSET_PREFIX}-${version}-linux-x86_64.tar.gz"
  say "versión $version"
  say "descargando binario y checksums"
  fetch "$base/${ASSET_PREFIX}-${version}-linux-x86_64.tar.gz" "$tarball"
  fetch "$base/SHA256SUMS.txt" "$work/SHA256SUMS.txt" \
    || die "no encontré SHA256SUMS.txt en v$version: la release no trae paquetes todavía"
  expected="$(grep -E "  ${ASSET_PREFIX}-${version}-linux-x86_64\.tar\.gz$" "$work/SHA256SUMS.txt" | cut -d' ' -f1)"
  [ -n "$expected" ] || die "el tarball no figura en SHA256SUMS.txt de v$version"
  actual="$(command -v sha256sum >/dev/null && sha256sum "$tarball" | cut -d' ' -f1 || shasum -a 256 "$tarball" | cut -d' ' -f1)"
  [ "$expected" = "$actual" ] || die "checksum no coincide (esperado $expected, obtenido $actual)"
  say "SHA-256 verificado"
fi

[ -n "$version" ] || die "no pude determinar la versión"
extract="$work/x"
mkdir -p "$extract"
tar -xzf "$tarball" -C "$extract"
src="$(find "$extract" -maxdepth 1 -type d -name "${ASSET_PREFIX}-*" | head -1)"
[ -n "$src" ] || die "el tarball no tiene el directorio esperado"

# --- plan ------------------------------------------------------------------
declare -a plan=(
  "$src/candil|$prefix/bin/candil|0755"
  "$src/candil.desktop|$prefix/share/applications/candil.desktop|0644"
  "$src/icons/256x256/apps/candil.png|$prefix/share/icons/hicolor/256x256/apps/candil.png|0644"
  "$src/icons/scalable/apps/candil.svg|$prefix/share/icons/hicolor/scalable/apps/candil.svg|0644"
  "$src/org.quicio.candil.metainfo.xml|$prefix/share/metainfo/org.quicio.candil.metainfo.xml|0644"
  "$src/candil.1.gz|$prefix/share/man/man1/candil.1.gz|0644"
  "$src/LICENSE|$prefix/share/licenses/candil/LICENSE|0644"
)

say "destino: $prefix"
for entry in "${plan[@]}"; do
  IFS='|' read -r from to perms <<<"$entry"
  [ -f "$from" ] || die "falta $from en el tarball"
  if [ -n "$dry_run" ]; then
    printf '  %s → %s (%s)\n' "$(basename "$from")" "$to" "$perms"
  else
    install -Dm"$perms" "$from" "$to"
  fi
done

# --- cachés del escritorio -------------------------------------------------
if [ -z "$dry_run" ]; then
  command -v update-desktop-database >/dev/null && \
    update-desktop-database "$prefix/share/applications" >/dev/null 2>&1 || true
  command -v gtk-update-icon-cache >/dev/null && \
    gtk-update-icon-cache -f -t "$prefix/share/icons/hicolor" >/dev/null 2>&1 || true
  command -v mandb >/dev/null && mandb -q >/dev/null 2>&1 || true
fi

echo
if [ -n "$dry_run" ]; then
  echo "dry-run: no se escribió nada."
  exit 0
fi

installed="$("$prefix/bin/candil" --version 2>/dev/null || true)"
echo "Candil instalado en $prefix ($installed)"
case ":$PATH:" in
  *":$prefix/bin:"*) ;;
  *)
    echo
    echo "Ojo: $prefix/bin no está en tu PATH. Agregá esto a tu shell:"
    echo "  export PATH=\"$prefix/bin:\$PATH\""
    ;;
esac
echo
echo "Abrilo con 'candil' o desde el menú de aplicaciones."
