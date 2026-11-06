#!/usr/bin/env bash
# Verificación local completa: lo mismo que la "definición de terminado".
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --profile fast
cargo run --profile fast -- --inspect tests/fixtures/mini.epub >/dev/null
cargo run --profile fast -- --inspect tests/fixtures/edge.epub >/dev/null

TMPHOME=$(mktemp -d)
HOME="$TMPHOME" ./target/fast/lector --scan tests/fixtures >/dev/null
rm -rf "$TMPHOME"

echo "ok: fmt + clippy + tests + fixtures + scan"
