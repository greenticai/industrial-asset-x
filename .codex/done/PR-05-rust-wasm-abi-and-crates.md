# PR 5 — Add Rust/WASM ABI and analytics crates

## Goal

Establish a Rust/WASM-first analytics foundation for industrial asset health, with no Python dependency.

## Scope

Add common crates and component ABI definitions for industrial analytics components.

## Adds crates

```text
crates/industrialx-core/
crates/industrialx-timeseries/
crates/industrialx-features/
crates/industrialx-baseline-models/
crates/industrialx-wasm-abi/
```

## Adds component ABI directories

```text
component_abi/feature-extractor/
component_abi/health-scorer/
component_abi/anomaly-detector/
component_abi/synthetic-generator/
component_abi/model-evaluator/
```

## Initial deterministic functions

```text
missing data detection
flatline detection
range checks
rolling mean
rolling standard deviation
rate of change
load-normalized temperature
simple residual calculation
control bands
basic anomaly score
feature vector serialization
```

## Acceptance criteria

- Crates build with Rust stable.
- WASM ABI is documented.
- No Python dependencies.
- Functions are deterministic and testable.
- Shared input/output structs match Industrial-X contracts where practical.

## Out of scope

- Full ML model training.
- GAN/diffusion generation.
- Customer-specific models.
