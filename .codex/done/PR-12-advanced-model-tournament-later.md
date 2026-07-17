# PR 12 — Add advanced model tournament skeleton later

## Goal

Prepare a Rust/WASM-first model tournament architecture for future predictive maintenance models.

## Scope

Add contracts and operation skeletons for model candidates, training runs, evaluations, selection, and registry.

## Adds contracts

```text
contracts/industrialx/training-dataset/
contracts/industrialx/model-candidate/
contracts/industrialx/training-run/
contracts/industrialx/model-selection-report/
```

## Adds ops

```text
ops/industrialx/prepare-training-dataset/
ops/industrialx/train-model-candidates/
ops/industrialx/evaluate-model-candidates/
ops/industrialx/select-model/
ops/industrialx/register-model/
```

## Rust/WASM-only constraint

Models should be implemented in Rust/WASM where possible. Acceptable early model families:

```text
statistical baselines
control bands
isolation-style deterministic anomaly scores
simple regression/residual models
Rust-native tree models if available and WASM-compatible
```

## Out of scope

- Python workers.
- External ML services.
- Deep learning training in MVP.
