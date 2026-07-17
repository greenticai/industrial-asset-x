# PR 7 — Add GX ops backed by WASM components

## Goal

Expose the generic WASM analytics components through Industrial-X GX operation manifests.

## Scope

Add operation packages with schemas, examples, and component references.

## Adds ops

```text
ops/industrialx/profile-signals/
ops/industrialx/validate-data-quality/
ops/industrialx/generate-features/
ops/industrialx/build-normal-behaviour-profile/
ops/industrialx/score-asset-health/
ops/industrialx/evaluate-shadow-performance/
```

## Acceptance criteria

- Each op has manifest, input schema, output schema, examples, permissions metadata, and component reference.
- Ops declare supported contracts.
- Ops produce evidence-compatible outputs.
- Ops are deterministic by default.

## Out of scope

- Agentic operations.
- Customer-specific flows.
- Store publication workflow.
