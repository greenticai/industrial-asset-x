# PR 16 — Replace component scaffolds with real WASM components

## Goal

Turn component package scaffolds into buildable WASM/WASI components with real input/output handling and fixture tests.

## Placeholder removal targets

```text
components/*/src/lib.rs simple helper functions only
components/*/component.manifest.json generic world/input/output declarations
component_abi/* README-only ABI docs
fixtures without automated fixture runner
```

## Scope

- Add WIT definitions or generated bindings for component worlds.
- Implement component entrypoints for:
  - Signal profiling.
  - Data quality checking.
  - Feature extraction.
  - Health scoring.
  - Rule-based synthetic generation.
- Add a shared fixture runner that loads `fixtures/input.json`, executes the component logic, and compares against `fixtures/expected-output.json`.
- Add `cargo component build --release --target wasm32-wasip2` validation to CI when tooling is available.

## Acceptance criteria

- Each component has real typed input/output structs.
- Fixtures are executed by automated tests.
- Component manifests reference the exact ABI/world and contracts.
- Components produce evidence-compatible outputs.
- CI documents or runs the WASM build path.

## Out of scope

- Customer-specific components.
- External ML services.
