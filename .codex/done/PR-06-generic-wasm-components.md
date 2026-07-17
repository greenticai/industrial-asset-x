# PR 6 — Add generic WASM components

## Goal

Provide first reusable Rust/WASM components for the MVP asset-health flow.

## Scope

Create generic WASM/WASI components that can be used by GX ops.

## Adds components

```text
components/generic-signal-profiler-wasm/
components/generic-data-quality-checker-wasm/
components/generic-feature-extractor-wasm/
components/generic-health-scorer-wasm/
```

## Each component includes

```text
Cargo.toml
component.manifest.json
src/lib.rs
fixtures/input.json
fixtures/expected-output.json
README.md
```

## Component responsibilities

### `generic-signal-profiler-wasm`

Computes summary statistics, coverage, min/max, missingness, flatline periods, and sample frequency.

### `generic-data-quality-checker-wasm`

Validates whether data is usable for baseline modelling.

### `generic-feature-extractor-wasm`

Generates rolling, lag, trend, and load-normalized features.

### `generic-health-scorer-wasm`

Produces basic health score and risk level using deterministic baseline scoring.

## Acceptance criteria

- Components build with `cargo component build --release --target wasm32-wasip2`.
- Component manifests use Greentic component schema/world.
- Components are generic and reusable.
- Fixture tests pass.

## Out of scope

- Demo presentation component.
- Koncar-specific WASM.
- Python or external ML services.
