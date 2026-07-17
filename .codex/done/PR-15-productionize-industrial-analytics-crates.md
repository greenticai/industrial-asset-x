# PR 15 — Productionize deterministic Industrial-X analytics crates

## Goal

Replace toy analytics helpers with robust deterministic implementations suitable for reusable industrial asset-health flows.

## Placeholder removal targets

```text
crates/industrialx-core minimal Sample/RangeCheck only
crates/industrialx-timeseries simple rolling helpers without timestamp semantics
crates/industrialx-features::feature_vector fixed three-value vector
crates/industrialx-baseline-models simplistic anomaly score
crates/industrialx-synthetic simple generated series
```

## Scope

- Add typed time-series models with units, timestamps, quality flags, and deterministic ordering.
- Implement missing-data, coverage, flatline, range, frequency, and outlier checks with documented edge-case behavior.
- Implement rolling statistics with timestamp-aware windows and NaN/None policies.
- Implement named feature sets with stable serialization, schema compatibility, and evidence.
- Implement baseline profile construction and health scoring with explainable residuals and risk levels.
- Implement rule-based synthetic scenarios with labels and expected failure-mode evidence.

## Acceptance criteria

- Public APIs are documented and tested.
- Edge cases are covered: empty data, all missing data, irregular timestamps, duplicate timestamps, NaN values, zero load, and short windows.
- Deterministic serialization is stable across platforms.
- Benchmarks target real hot paths instead of placeholder summation.
- All crates remain Python-free and build on stable Rust.

## Out of scope

- Deep learning or external ML services.
- Customer-specific models.
