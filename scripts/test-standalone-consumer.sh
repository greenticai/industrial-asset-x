#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/industrialx-consumer.XXXXXX")"
trap 'rm -rf "$TEMP_DIR"' EXIT

workspace_value() {
  local key="$1"
  sed -n "/^\[workspace.package\]/,/^\[/s/^${key} = \"\(.*\)\"/\1/p" "$ROOT_DIR/Cargo.toml"
}

WORKSPACE_VERSION="$(workspace_value version)"
WORKSPACE_EDITION="$(workspace_value edition)"
WORKSPACE_RUST_VERSION="$(workspace_value rust-version)"
WORKSPACE_LICENSE="$(workspace_value license)"
WORKSPACE_REPOSITORY="$(workspace_value repository)"
WORKSPACE_HOMEPAGE="$(workspace_value homepage)"

mkdir -p "$TEMP_DIR/crates" "$TEMP_DIR/consumer/src"
for crate in industrialx-core industrialx-timeseries industrialx-features industrialx-baseline-models; do
  cp -R "$ROOT_DIR/crates/$crate" "$TEMP_DIR/crates/$crate"
done
cp "$ROOT_DIR/LICENSE" "$TEMP_DIR/LICENSE"

cat > "$TEMP_DIR/Cargo.toml" <<EOF
[workspace]
members = [
  "crates/industrialx-core",
  "crates/industrialx-timeseries",
  "crates/industrialx-features",
  "crates/industrialx-baseline-models",
]
exclude = ["consumer"]
resolver = "3"

[workspace.package]
version = "$WORKSPACE_VERSION"
edition = "$WORKSPACE_EDITION"
rust-version = "$WORKSPACE_RUST_VERSION"
license = "$WORKSPACE_LICENSE"
repository = "$WORKSPACE_REPOSITORY"
homepage = "$WORKSPACE_HOMEPAGE"

[workspace.dependencies]
industrialx-timeseries = { version = "$WORKSPACE_VERSION", path = "crates/industrialx-timeseries" }
EOF

cat > "$TEMP_DIR/consumer/Cargo.toml" <<EOF
[package]
name = "industrialx-standalone-consumer"
version = "0.0.0"
edition = "$WORKSPACE_EDITION"
publish = false

[workspace]

[dependencies]
industrialx-core = { path = "../crates/industrialx-core" }
industrialx-timeseries = { path = "../crates/industrialx-timeseries" }
industrialx-features = { path = "../crates/industrialx-features" }
industrialx-baseline-models = { path = "../crates/industrialx-baseline-models" }
EOF

cat > "$TEMP_DIR/consumer/src/main.rs" <<'EOF'
use industrialx_baseline_models::ThermalBaseline;
use industrialx_core::{coverage_ratio, Quality, Sample};
use industrialx_features::residual;
use industrialx_timeseries::rolling_mean;

fn main() {
    let samples = [Sample {
        timestamp_ms: 1,
        value: Some(42.0),
        quality: Quality::Good,
    }];
    assert_eq!(coverage_ratio(&samples), 1.0);
    assert_eq!(rolling_mean(&[1.0, 3.0], 2), vec![1.0, 2.0]);
    assert_eq!(residual(12.0, 10.0), 2.0);
    let model = ThermalBaseline {
        no_load_rise_c: 20.0,
        load_rise_c: 30.0,
        cooling_relief_c: 8.0,
        residual_band_c: 4.0,
    };
    assert!(model.assess(58.0, 25.0, 0.8, 1.0).is_ok());
}
EOF

cargo run --quiet --manifest-path "$TEMP_DIR/consumer/Cargo.toml"
