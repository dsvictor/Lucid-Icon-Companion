#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

command -v cargo >/dev/null || { echo "Rust/Cargo is required. Install it with https://rustup.rs"; exit 1; }
command -v npm >/dev/null || { echo "Node.js/npm is required to fetch the Lucide catalogue."; exit 1; }

echo "Downloading the official lucide-static package…"
npm pack lucide-static --pack-destination "$TMP" >/dev/null
tar -xzf "$TMP"/lucide-static-*.tgz -C "$TMP"

python3 "$ROOT/scripts/generate_catalogue.py" "$TMP/package/icons" "$ROOT/catalogue.json"

rustup target add wasm32-unknown-unknown
cargo build --manifest-path "$ROOT/Cargo.toml" --release --target wasm32-unknown-unknown

DIST="$ROOT/dist"
mkdir -p "$DIST"
cp "$ROOT/figcraft-plugin.json" "$DIST/"
cp "$ROOT/target/wasm32-unknown-unknown/release/lucide_companion.wasm" "$DIST/"
cp "$ROOT/catalogue.json" "$DIST/"
echo "Plugin bundle ready in: $DIST"
