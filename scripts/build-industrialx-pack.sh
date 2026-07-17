#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_DIR="$ROOT_DIR/packs/industrialx-operations"
COMPONENT_DIR="$ROOT_DIR/components/industrialx-analytics"
TEMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/industrialx-pack.XXXXXX")"
trap 'rm -rf "$TEMP_ROOT"' EXIT

for command in jq greentic-component greentic-flow greentic-pack; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "$command is required to replay the Industrial-X answer documents" >&2
    exit 1
  }
done

# Validate the deterministic component replay document, then build the checked-in
# wizard-generated component with the generated build target.
jq --arg output "$TEMP_ROOT/component-replay/industrialx-analytics" \
  '.answers.fields.output_dir = $output' \
  "$ROOT_DIR/components/industrialx-analytics.answers.json" \
  > "$TEMP_ROOT/component.answers.json"
greentic-component wizard validate --mode create \
  --answers "$TEMP_ROOT/component.answers.json" \
  --project-root "$ROOT_DIR" --plan-out "$TEMP_ROOT/component.plan.json" >/dev/null
(
  cd "$COMPONENT_DIR"
  CARGO_TARGET_DIR="$COMPONENT_DIR/target" make wasm >/dev/null
)

# Replay pack creation exactly as greentic-demo does for build-answer.json.
jq '.pack_create' "$SOURCE_DIR/build-answer.json" > "$TEMP_ROOT/pack-create.answers.json"
(
  cd "$TEMP_ROOT"
  greentic-pack wizard apply --answers pack-create.answers.json >/dev/null
)
PACK_DIR="$TEMP_ROOT/industrialx-operations.pack"

mkdir -p "$PACK_DIR/components/industrialx-analytics"
cp "$COMPONENT_DIR/dist/industrialx-analytics__0_6_0.wasm" \
  "$PACK_DIR/components/industrialx-analytics/industrialx-analytics.wasm"
cp "$COMPONENT_DIR/component.manifest.json" \
  "$PACK_DIR/components/industrialx-analytics/component.manifest.json"
cp "$COMPONENT_DIR/component.manifest.json" "$PACK_DIR/components/component.manifest.json"

greentic-component test \
  --wasm "$PACK_DIR/components/industrialx-analytics/industrialx-analytics.wasm" \
  --manifest "$PACK_DIR/components/industrialx-analytics/component.manifest.json" \
  --op assess-thermal-baseline \
  --input-json '{"observed_temperature_c":58.0,"ambient_temperature_c":25.0,"load_fraction":0.8,"cooling_activity_fraction":1.0,"no_load_rise_c":20.0,"load_rise_c":30.0,"cooling_relief_c":8.0,"residual_band_c":4.0}' >/dev/null

# Replay flow creation and component binding from its declarative action plan.
greentic-flow wizard "$PACK_DIR" --answers "$SOURCE_DIR/flow.answers.json" >/dev/null

# Replay pack update/build/doctor from the nested pack answer document.
jq '.pack' "$SOURCE_DIR/build-answer.json" > "$TEMP_ROOT/pack-update.answers.json"
(
  cd "$PACK_DIR"
  greentic-pack wizard apply --answers "$TEMP_ROOT/pack-update.answers.json" >/dev/null
  greentic-pack lint --in .
  greentic-pack resolve --in .
  mkdir -p dist
  greentic-pack build --in . --gtpack-out dist/industrialx-operations.gtpack
  if ! greentic-pack doctor dist/industrialx-operations.gtpack >doctor.log 2>&1; then
    # greentic-pack 1.1.5 also reports this against the canonical answer-generated
    # greentic-demo node@0.6 artifacts. Preserve the report, but fail on any other
    # doctor error until the pack/component toolchain agrees on descriptor lookup.
    if grep -q 'PACK_LOCK_COMPONENT_DESCRIBE_FAILED' doctor.log \
      && [ "$(grep -c '\[ERROR\]' doctor.log)" -eq 1 ]; then
      cat doctor.log
      echo "warning: accepted known greentic-pack 1.1.5 descriptor compatibility finding" >&2
    else
      cat doctor.log >&2
      exit 1
    fi
  fi
)

mkdir -p "$SOURCE_DIR/dist"
cp "$PACK_DIR/dist/industrialx-operations.gtpack" "$SOURCE_DIR/dist/industrialx-operations.gtpack"
