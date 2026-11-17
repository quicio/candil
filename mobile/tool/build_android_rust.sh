#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/rust"

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "Falta cargo-ndk. Instalalo con: cargo install cargo-ndk" >&2
  exit 1
fi
if [[ -z "${ANDROID_NDK_HOME:-}" && -z "${ANDROID_NDK_ROOT:-}" ]]; then
  echo "Definí ANDROID_NDK_HOME (o ANDROID_NDK_ROOT) antes de compilar." >&2
  exit 1
fi

rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -o "$ROOT/android/app/src/main/jniLibs" \
  build --release

echo "Listo: android/app/src/main/jniLibs"
