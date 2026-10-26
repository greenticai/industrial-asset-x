#!/usr/bin/env bash
# Build the Industrial-X Designer design extension into a `.gtxpack`.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE"

echo "==> cargo component build"
cargo component build --release --target wasm32-wasip2

# cargo-component writes the component under wasip2 or wasip1 depending on its
# version; honour CARGO_TARGET_DIR and take whichever it produced.
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
WASM_PATH=""
for triple in wasm32-wasip2 wasm32-wasip1; do
  candidate="$TARGET_DIR/$triple/release/greentic_industrial_x_designer_extension.wasm"
  if [ -f "$candidate" ]; then WASM_PATH="$candidate"; break; fi
done
if [ -z "$WASM_PATH" ]; then
  echo "ERROR: wasm not found under $TARGET_DIR/wasm32-wasip{2,1}/release" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cp describe.json "$STAGE/"
# Shipped beside the extension so a host can serve it; describe.json declares no key for it.
cp ../../playbooks/playbook.schema.json "$STAGE/playbook.schema.json"
cp "$WASM_PATH" "$STAGE/extension.wasm"

WASM_SHA256="$(shasum -a 256 "$STAGE/extension.wasm" | cut -d' ' -f1)"
sed "s/0000000000000000000000000000000000000000000000000000000000000000/${WASM_SHA256}/g"   "$STAGE/describe.json" > "$STAGE/describe.json.tmp"
mv "$STAGE/describe.json.tmp" "$STAGE/describe.json"

EXT_ID="$(jq -r .metadata.id "$STAGE/describe.json")"
EXT_VERSION="$(jq -r .metadata.version "$STAGE/describe.json")"
OUT="$HERE/${EXT_ID}-${EXT_VERSION}.gtxpack"
TMP_ZIP="$STAGE/../greentic_industrial_x_designer_$$.zip"
(cd "$STAGE" && zip -r "$TMP_ZIP" .) > /dev/null
mv "$TMP_ZIP" "$OUT"

echo "==> built $OUT"
echo "==> size: $(du -h "$OUT" | cut -f1)"
