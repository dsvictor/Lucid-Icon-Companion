#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

LUCIDE_VERSION="${LUCIDE_VERSION:-1.72.0}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

for tool in cargo rustup npm python3; do
  command -v "$tool" >/dev/null 2>&1 || {
    echo "Missing required tool: $tool" >&2
    exit 1
  }
done

echo "Downloading lucide-static@${LUCIDE_VERSION}…"
npm pack --ignore-scripts "lucide-static@${LUCIDE_VERSION}" --pack-destination "$TMP" >/dev/null
tar -xzf "$TMP"/lucide-static-*.tgz -C "$TMP"

ICONS_DIR="$TMP/package/icons"
if [ ! -d "$ICONS_DIR" ]; then
  echo "The lucide-static package did not contain package/icons." >&2
  exit 1
fi

python3 "$ROOT/scripts/generate_catalogue.py" "$ICONS_DIR" "$ROOT/catalogue.json"
python3 -m unittest discover -s "$ROOT/tests" -p 'test_*.py'

echo "Running Rust unit tests…"
cargo test --manifest-path "$ROOT/Cargo.toml"

echo "Installing the WebAssembly target…"
rustup target add wasm32-unknown-unknown

echo "Building Lucide Companion for FigCraft…"
cargo build --manifest-path "$ROOT/Cargo.toml" --release --target wasm32-unknown-unknown

DIST="$ROOT/dist"
rm -rf "$DIST"
mkdir -p "$DIST/assets"
cp "$ROOT/figcraft-plugin.json" "$DIST/"
cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/lucide_companion.wasm" "$DIST/"
cp "$ROOT/licenses/LUCIDE-LICENSE.txt" "$DIST/assets/"

echo "Plugin bundle ready in: $DIST"
