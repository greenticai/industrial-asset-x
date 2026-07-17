# PR 11 — Add synthetic scenario support later

## Goal

Introduce synthetic scenario support after the deterministic baseline flow is working.

## Scope

Add contracts, ops, and Rust/WASM rule-based generator for synthetic scenario stress testing.

## Adds contracts

```text
contracts/industrialx/synthetic-scenario/
contracts/industrialx/synthetic-dataset/
```

## Adds crate/component

```text
crates/industrialx-synthetic/
components/rule-based-synthetic-generator-wasm/
```

## Adds ops

```text
ops/industrialx/generate-synthetic-scenarios/
ops/industrialx/validate-synthetic-scenarios/
ops/industrialx/evaluate-model-against-synthetic-scenarios/
```

## Initial scenarios

```text
gradual temperature drift
cooling fan degradation
sensor flatline
current imbalance trend
load-normalized overheating
```

## Important MVP principle

Synthetic data is used for **stress testing first**, not primary training.

## Out of scope

- GANs.
- Diffusion models.
- Training production models on synthetic data.
