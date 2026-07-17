# PR 17 — Productionize GX ops, flows, and neutral views

## Goal

Replace permissive operation, flow, and view descriptors with executable, schema-checked definitions that can be reused by demo packs and GX runtimes.

## Placeholder removal targets

```text
ops/industrialx/** generic input/output schemas
ops/industrialx/** generic examples
flows/** single-step placeholder flow definitions
views/** minimal neutral-json descriptors
```

## Scope

- Add strict input/output schemas for each operation.
- Add realistic synthetic examples that exercise each operation.
- Define permissions, deterministic behavior, component versions, and supported contracts precisely.
- Expand flows into full ordered step graphs with inputs, outputs, error handling, and evidence propagation.
- Define view models with sections, fields, actions, filters, and contract bindings.

## Acceptance criteria

- Every op schema validates examples and rejects invalid fixtures.
- Flows connect real op outputs to downstream inputs.
- Views bind to contract fields rather than generic section names only.
- Tests validate all operation/flow/view JSON.
- No Webchat-specific or customer-specific assumptions appear.

## Out of scope

- Runtime UI rendering implementation.
- Agentic operations.
