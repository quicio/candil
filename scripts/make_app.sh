#!/usr/bin/env bash
# Arma un Candil.app de macOS alrededor de un binario ya compilado.
#
# Uso:
#   scripts/make_app.sh [binario] [salida]
#
# Por defecto usa target/dist/candil (o ~/.local/bin/candil si no existe) y
# escribe target/Candil.app. Para instalarlo en el usuario:
#   scripts/make_app.sh ~/.local/bin/candil "$HOME/Applications/Candil.app"
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$PWD"
BIN="${1:-$ROOT/target/dist/candil}"
if [ ! -x "$BIN" ]; then
  if [ -x "$HOME/.local/bin/candil" ]; then
    BIN="$HOME/.local/bin/candil"
  else
    echo "falta el binario: $BIN" >&2
    echo "compilá con: cargo build --profile dist" >&2
    exit 1
  fi
fi
OUT="${2:-$ROOT/target/Candil.app}"
VERSION="${CANDIL_VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)}"
VERSION="${VERSION:-0.2.1}"

if [ ! -f "$ROOT/assets/candil.icns" ]; then
  echo "falta assets/candil.icns" >&2
  exit 1
fi

rm -rf "$OUT"
mkdir -p "$OUT/Contents/MacOS" "$OUT/Contents/Resources"
cp "$BIN" "$OUT/Contents/MacOS/candil"
chmod 755 "$OUT/Contents/MacOS/candil"
cp "$ROOT/assets/candil.icns" "$OUT/Contents/Resources/Candil.icns"

cat > "$OUT/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>Candil</string>
  <key>CFBundleDisplayName</key>
  <string>Candil</string>
  <key>CFBundleIdentifier</key>
  <string>com.quicio.candil</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleExecutable</key>
  <string>candil</string>
  <key>CFBundleIconFile</key>
  <string>Candil</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.books</string>
</dict>
</plist>
PLIST

chmod 644 "$OUT/Contents/Info.plist"
# Firma ad-hoc: evita que Gatekeeper trate el bundle como dañado.
if command -v codesign >/dev/null 2>&1; then
  codesign --force --deep --sign - "$OUT" >/dev/null 2>&1 || true
fi
xattr -dr com.apple.quarantine "$OUT" 2>/dev/null || true

echo "app -> $OUT (Candil $VERSION)"
