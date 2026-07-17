# PR 20 — Productionize synthetic scenarios and model tournament skeleton

## Goal

Replace future-facing synthetic scenario and model tournament skeletons with validated, deterministic production foundations.

## Placeholder removal targets

```text
contracts/industrialx/synthetic-*/contract.json broad schemas
contracts/industrialx/model-*/contract.json broad schemas
ops/industrialx/*synthetic* permissive ops
ops/industrialx/*model* permissive ops
crates/industrialx-synthetic simple series generator
```

## Scope

- Define strict synthetic scenario and synthetic dataset contracts.
- Define strict training dataset, model candidate, training run, model selection report, and evaluation contracts.
- Implement deterministic rule-based scenario generation with labels, expected outcomes, and evidence.
- Implement model tournament orchestration schemas and deterministic baseline candidate evaluation.
- Add tests that compare candidates on synthetic stress scenarios.

## Acceptance criteria

- Synthetic scenarios are used for stress testing first, not primary training.
- Model tournament operations remain Rust/WASM-first and Python-free.
- Evaluation outputs include deterministic metrics, selection evidence, and tie-breaking rules.
- All examples and invalid fixtures are schema-tested.

## Out of scope

- GANs, diffusion models, or deep learning training.
- External model-serving infrastructure.
