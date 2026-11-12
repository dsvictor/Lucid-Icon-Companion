#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
LUCIDE_VERSION="${LUCIDE_VERSION:-1.72.0}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
for tool in cargo rustup npm node python3; do
  command -v "$tool" >/dev/null 2>&1 || { echo "Missing required tool: $tool" >&2; exit 1; }
done
echo "Downloading Lucide SVGs and the SVG preview renderer…"
npm pack --ignore-scripts "lucide-static@${LUCIDE_VERSION}" --pack-destination "$TMP" >/dev/null
tar -xzf "$TMP"/lucide-static-*.tgz -C "$TMP"
npm install --prefix "$TMP/tools" --no-save --no-audit --no-fund @resvg/resvg-js
ICONS_DIR="$TMP/package/icons"
[ -d "$ICONS_DIR" ] || { echo "Lucide package icons directory missing" >&2; exit 1; }
python3 "$ROOT/scripts/generate_catalogue.py" "$ICONS_DIR" "$ROOT/catalogue.json"
RESVG_MODULE="$TMP/tools/node_modules/@resvg/resvg-js/index.js" node "$ROOT/scripts/render_previews.mjs" "$ROOT/catalogue.json"
python3 -m unittest discover -s "$ROOT/tests" -p 'test_*.py'
cargo test --manifest-path "$ROOT/Cargo.toml"
rustup target add wasm32-unknown-unknown
cargo build --manifest-path "$ROOT/Cargo.toml" --release --target wasm32-unknown-unknown
DIST="$ROOT/dist"
rm -rf "$DIST"
mkdir -p "$DIST/assets"
cp "$ROOT/figcraft-plugin.json" "$DIST/"
cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/lucide_companion.wasm" "$DIST/"
cp "$ROOT/licenses/LUCIDE-LICENSE.txt" "$DIST/assets/"
echo "Plugin bundle ready in: $DIST"
