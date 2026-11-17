#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKSPACE="$(cd "$ROOT/.." && pwd)"
TARGET="$WORKSPACE/target"
cd "$ROOT/rust"

# Requiere macOS con Xcode y los targets de Rust para iOS.
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

mkdir -p "$ROOT/ios/Frameworks"
DEVICE="$TARGET/aarch64-apple-ios/release/libcandil_mobile_core.a"
SIM_ARM="$TARGET/aarch64-apple-ios-sim/release/libcandil_mobile_core.a"
SIM_X64="$TARGET/x86_64-apple-ios/release/libcandil_mobile_core.a"
SIM_UNIVERSAL="$TARGET/ios-sim-universal.a"
lipo -create "$SIM_ARM" "$SIM_X64" -output "$SIM_UNIVERSAL"

rm -rf "$ROOT/ios/Frameworks/CandilCore.xcframework"
xcodebuild -create-xcframework \
  -library "$DEVICE" \
  -library "$SIM_UNIVERSAL" \
  -output "$ROOT/ios/Frameworks/CandilCore.xcframework"

echo "Listo: ios/Frameworks/CandilCore.xcframework"
echo "En Xcode: agregalo a Runner, marcá Embed & Sign y en Other Linker Flags"
echo "agregá -force_load \$(PROJECT_DIR)/Frameworks/CandilCore.xcframework/ios-arm64/libcandil_mobile_core.a"
