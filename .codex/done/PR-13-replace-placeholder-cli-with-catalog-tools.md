# PR 13 — Replace placeholder CLI with production catalog tools

## Goal

Replace the `industrial-asset-x placeholder result` binary with useful repository tooling for validating and inspecting the catalog.

## Placeholder removal targets

```text
src/main.rs
src/lib.rs::example_hot_path
benches/perf.rs placeholder benchmark target
tests/perf_scaling.rs placeholder workload dependency
```

## Scope

Add a small production CLI with subcommands:

```text
industrial-asset-x catalog summary
industrial-asset-x catalog validate
industrial-asset-x store validate <package-dir>
industrial-asset-x contracts list
industrial-asset-x components list
```

## Implementation notes

- Use structured parsing and validation, not ad hoc string checks.
- Reuse local validation logic from the store validator rather than duplicating it in shell.
- Output stable JSON by default, with a concise human format behind a flag if needed.
- Keep the binary deterministic and offline.

## Acceptance criteria

- Running the binary no longer prints placeholder arithmetic.
- `catalog validate` verifies `catalog.json`, contract files, operation manifests, flows, views, playbooks, component manifests, and store examples.
- `store validate` covers at least the checks currently in `scripts/validate-store-package.sh`.
- Tests cover success and failure cases.
- `ci/local_check.sh` uses the Rust validator instead of relying only on shell checks.

## Out of scope

- Networked store publication.
- Customer-specific demo validation.
