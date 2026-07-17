# PR 24 — Publish an executable Greentic Industrial-X operation pack

## Goal

Turn the existing Rust/WASM library scaffolds, operation manifests, and JSON flow
descriptors into versioned Greentic components and a resolvable pack that application
flows can execute with `component.exec` or the standard GX operation mechanism.

## Dependency

Complete PR 23 first.

## Current limitation

The crates under `components/` compile Rust functions and can target WASI, but they do
not implement the Greentic component world used by application packs. The manifests
under `ops/` and flows under `flows/` are catalog metadata rather than artifacts that
`koncar-demo` can currently resolve and invoke at runtime.

## Scope

- Implement the supported Greentic component ABI for signal profiling, data-quality
  validation, feature generation, load-aware thermal baseline assessment, and health
  scoring.
- Reuse shared Greentic interfaces/types and the published Industrial-X Rust crates;
  do not define a parallel invocation envelope or serialization convention.
- Define closed, versioned input/output schemas that align operation manifests,
  component descriptors, fixtures, and runtime payloads.
- Ensure the thermal operation accepts raw mapped operating inputs and returns
  expected temperature, residual, health score, risk, and evidence/error diagnostics.
- Package the components plus `validate-industrial-data`,
  `build-asset-health-baseline`, and `run-shadow-health-check` into a versioned
  Greentic pack with resolvable component ids and flow entrypoints.
- Publish the pack/component artifacts with immutable versions/digests and provenance.
- Add native fixture tests, built-WASM invocation tests, pack lint/resolve/doctor
  checks, and an end-to-end flow test.
- Document capability requirements, deterministic behavior, resource bounds, errors,
  compatibility, and upgrade policy.

## Acceptance criteria

- A separate Greentic application can consume the published pack without copying
  Industrial-X WASM or manifests.
- Built-WASM tests invoke exported Greentic operations rather than native Rust helper
  functions only.
- The packaged shadow-health flow produces a typed assessment from raw mapped sample
  input and propagates evidence across steps.
- Invalid contracts, poor data quality, and non-finite inputs return structured
  diagnostics.
- Components declare minimal capabilities and require no network/filesystem access for
  deterministic analytics.
- Pack artifacts pass lint, resolve, build, and doctor checks.
- `.codex/repo_overview.md` is refreshed and `bash ci/local_check.sh` passes.

## Out of scope

- Customer-specific cards, mappings, tags, or branding.
- Telemetry transport and durable state providers.
- Automated maintenance-system actions.

