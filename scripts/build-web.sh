#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TARGET="wasm32-unknown-unknown"
WASM_NAME="snake-rs.wasm"
WASM_PATH="$ROOT_DIR/target/$TARGET/release/$WASM_NAME"
ALT_WASM_PATH="$ROOT_DIR/target/$TARGET/release/snake_rs.wasm"

if command -v rustup >/dev/null 2>&1; then
  rustup target add "$TARGET" >/dev/null
fi

cargo build --release --target "$TARGET"

if [[ ! -f "$WASM_PATH" && -f "$ALT_WASM_PATH" ]]; then
  WASM_PATH="$ALT_WASM_PATH"
fi

if [[ ! -f "$WASM_PATH" ]]; then
  echo "WASM build artifact not found at expected path" >&2
  exit 1
fi

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

cp "$WASM_PATH" "$DIST_DIR/$WASM_NAME"
cp "$ROOT_DIR/vendor/miniquad/js/gl.js" "$DIST_DIR/gl.js"
cp -R "$ROOT_DIR/assets" "$DIST_DIR/assets"
cp "$ROOT_DIR/web/index.html" "$DIST_DIR/index.html"

echo "Web bundle ready at $DIST_DIR"
