# PR 23 — Publish supported Industrial-X Rust crate boundaries

## Goal

Provide a reproducible, versioned dependency surface so downstream repositories can
reuse Industrial-X without sibling `path` dependencies or copied source.

## Why

`industrialx-core`, `industrialx-timeseries`, `industrialx-features`, and
`industrialx-baseline-models` are all `publish = false`. `koncar-demo` therefore
requires a specific sibling checkout layout and cannot independently build in CI or
from a released source archive.

## Scope

- Decide and document the supported release boundary: independently published crates
  or a consolidated public facade crate. Prefer the smallest stable public surface.
- Publish the core, time-series, feature, and thermal-baseline APIs required by
  downstream consumers, including `ThermalBaseline` and its error/result types.
- Replace internal path-only dependency metadata with publishable version constraints
  while retaining workspace path resolution for local development where supported by
  Cargo.
- Add public API documentation, semver policy, MSRV, changelog/release notes, and
  ownership/support expectations.
- Add downstream compatibility testing using a temporary standalone consumer crate
  that has no path reference into this repository.
- Ensure package contents contain every required source/license/readme file and no
  catalog/demo assets that do not belong in the crate release.
- Update the release workflow to publish in dependency order and fail safely on a
  partial release.

## Acceptance criteria

- A standalone consumer can depend on a tagged/released Industrial-X version and call
  data-quality, feature, and thermal-baseline APIs.
- `cargo package` and `cargo publish --dry-run` pass for every public crate in release
  order.
- Published crates contain no customer-specific content.
- Public types and error behavior have API documentation and semver coverage.
- The repository documents exact downstream dependency syntax and upgrade policy.
- `.codex/repo_overview.md` is refreshed and `bash ci/local_check.sh` passes.

## Out of scope

- Greentic component ABI and pack publication; handled by PR 24.
- Vendor telemetry adapters.

