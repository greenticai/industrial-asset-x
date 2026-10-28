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
# Dialect service address read by the designer (exactly {"url": ...}); it is only
# used when it matches a `runtime.permissions.network` pattern in describe.json.
mkdir -p "$STAGE/assets"
cp assets/dialect-service.json "$STAGE/assets/dialect-service.json"

WASM_SHA256="$(shasum -a 256 "$STAGE/extension.wasm" | cut -d' ' -f1)"
sed "s/0000000000000000000000000000000000000000000000000000000000000000/${WASM_SHA256}/g"   "$STAGE/describe.json" > "$STAGE/describe.json.tmp"
mv "$STAGE/describe.json.tmp" "$STAGE/describe.json"

# manifest.json: per-file sha256 + size, the shape `gtdx publish` writes
# (schema greentic.gtxpack.manifest/v1). Hosts verify entries against it on
# install. It must be written last, after every other staged file is final.
(cd "$STAGE" && for f in $(find . -type f ! -name manifest.json | sed 's|^\./||' | LC_ALL=C sort); do
  printf '%s\t%s\t%s\n' "$f" "$(shasum -a 256 "$f" | cut -d' ' -f1)" "$(wc -c < "$f" | tr -d ' ')"
done) | jq -R -s -c '{entries: (split("\n") | map(select(length>0) | split("\t") | {path: .[0], sha256: .[1], size: (.[2]|tonumber)})), schema: "greentic.gtxpack.manifest/v1"}' > "$STAGE/manifest.json"

EXT_ID="$(jq -r .metadata.id "$STAGE/describe.json")"
EXT_VERSION="$(jq -r .metadata.version "$STAGE/describe.json")"
OUT="$HERE/${EXT_ID}-${EXT_VERSION}.gtxpack"
TMP_ZIP="$STAGE/../greentic_industrial_x_designer_$$.zip"
(cd "$STAGE" && zip -r "$TMP_ZIP" .) > /dev/null
mv "$TMP_ZIP" "$OUT"

echo "==> built $OUT"
echo "==> size: $(du -h "$OUT" | cut -f1)"
