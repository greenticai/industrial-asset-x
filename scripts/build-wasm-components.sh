#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v cargo-component >/dev/null 2>&1; then
  echo "cargo-component is required for WASM component validation" >&2
  echo "install it with: cargo install cargo-component" >&2
  exit 1
fi

if ! rustup target list --installed | grep -q '^wasm32-wasip2$'; then
  echo "wasm32-wasip2 target is required for WASM component validation" >&2
  echo "install it with: rustup target add wasm32-wasip2" >&2
  exit 1
fi

components="
generic-signal-profiler-wasm
generic-data-quality-checker-wasm
generic-feature-extractor-wasm
generic-health-scorer-wasm
rule-based-synthetic-generator-wasm
industrialx-analytics
"

for component in $components; do
  printf '\n==> Build WASM component %s\n' "$component"
  answers="components/$component.answers.json"
  if [ ! -f "$answers" ]; then
    echo "missing component replay document: $answers" >&2
    exit 1
  fi
  (
    cd "components/$component"
    CARGO_TARGET_DIR="$ROOT_DIR/components/$component/target" make wasm
  )
done
