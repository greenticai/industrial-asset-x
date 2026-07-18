#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mode="${1:-all}"

header() {
  printf '\n==> %s\n' "$1"
}

dirty_flag() {
  if [ -z "$(git status --porcelain)" ]; then
    return 0
  fi
  printf '%s\n' "--allow-dirty"
}

detect_release_packages() {
  if command -v python3 >/dev/null 2>&1; then
    cargo metadata --no-deps --format-version 1 | python3 -c '
import json
import sys

metadata = json.load(sys.stdin)
workspace_members = set(metadata.get("workspace_members", []))
order = {
    "industrialx-timeseries": 10,
    "industrialx-core": 20,
    "industrialx-baseline-models": 30,
    "industrialx-features": 40,
    "industrial-asset-x": 100,
}
packages = sorted(metadata.get("packages", []), key=lambda item: (order.get(item["name"], 50), item["name"]))
for package in packages:
    if package.get("id") not in workspace_members:
        continue
    if package["name"] in order:
        print(package["name"])
'
  else
    cargo metadata --no-deps --format-version 1 >/dev/null
    sed -n 's/^name = "\(.*\)"/\1/p' Cargo.toml | head -n 1
  fi
}

package_checks() {
  header "Detect release packages"
  crates_file="$(mktemp "${TMPDIR:-/tmp}/release-packages.XXXXXX")"
  trap 'rm -f "$crates_file"' EXIT
  detect_release_packages > "$crates_file"

  if [ ! -s "$crates_file" ]; then
    printf 'No release packages found.\n'
    return 0
  fi

  while IFS= read -r crate; do
    [ -n "$crate" ] || continue
    header "Package $crate without verification"
    cargo package --offline --no-verify -p "$crate" $(dirty_flag)

    header "Package $crate with verification"
    cargo package --offline -p "$crate" $(dirty_flag)

    header "List packaged files for $crate"
    cargo package --offline --list -p "$crate" $(dirty_flag)

  done < "$crates_file"
}

if [ "$mode" = "package-only" ]; then
  package_checks
  exit 0
fi

header "Format"
cargo fmt --all -- --check

header "Clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

header "Test"
cargo test --workspace --all-features

header "Store package validation"
bash scripts/validate-store-package.sh store/examples/substation-transformer-health-basic

header "WASM component builds"
bash scripts/build-wasm-components.sh

header "Build"
cargo build --workspace --all-features

header "Docs"
cargo doc --workspace --no-deps --all-features

header "Standalone public crate consumer"
bash scripts/test-standalone-consumer.sh

header "Industrial-X Greentic operation pack"
bash scripts/build-industrialx-pack.sh

package_checks
