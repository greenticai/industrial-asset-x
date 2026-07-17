# industrial-asset-x

`industrial-asset-x` is the generic Rust/WASM-first Asset-X and Industrial-X catalog foundation for industrial asset health.

The current crate is intentionally minimal while the repository structure and contracts are being built out.

## Catalog Scope

This repository is a reusable Greentic-X catalog foundation. It defines generic namespaces for:

- `assetx.*` reusable industrial asset, component, sensor, failure-mode, maintenance, and health resources.
- `industrialx.*` deterministic industrial analytics, baseline, prediction, recommendation, feedback, synthetic scenario, and model evaluation resources.
- `storex.*` package, publisher, license, component, playbook listing, validation, and compatibility metadata.

The repository must remain customer-neutral. Demo repositories may wrap these contracts and components for a specific presentation, but customer names, customer data, private tags, and customer-specific branding do not belong here.

## Repository Layout

- `contracts/` contains machine-readable resource contracts and schemas.
- `ops/` contains GX operation manifests backed by deterministic Rust/WASM components.
- `resolvers/` is reserved for namespace and resource resolver definitions.
- `flows/` composes operations into reusable GX flows.
- `views/` contains neutral view definitions.
- `playbooks/` contains reusable industrial health playbooks.
- `component_abi/` documents component interfaces.
- `crates/` contains Rust analytics and ABI crates.
- `components/` contains reusable Rust/WASM component packages.
- `store/` contains package schemas and reusable package examples.
- `docs/` contains architecture, naming, contribution, promotion, and quality-gate documentation.

## CI and Releases

Run the same checks locally that CI runs:

```sh
bash ci/local_check.sh
```

The local check runs formatting, clippy, tests, build, documentation, package validation, and crates.io dry-run publishing checks for publishable crates.

It also builds all reusable component packages with:

```sh
bash scripts/build-wasm-components.sh
```

That command requires `cargo-component` and the `wasm32-wasip2` Rust target.

Pull requests and pushes run `.github/workflows/ci.yml`, which splits linting, tests, and package dry-runs into separate jobs.

Releases are driven by the version in `Cargo.toml`:

1. Bump `version` in `Cargo.toml`.
2. Commit the change.
3. Create and push a matching tag, for example `v0.1.0`.
4. The `publish.yml` workflow verifies that the tag matches the Cargo version, runs `ci/local_check.sh`, performs another publish dry-run, publishes to crates.io, and creates a GitHub Release.

Required secret:

- `CARGO_REGISTRY_TOKEN` for crates.io publishing.

## Versioning

The root crate version represents a catalog snapshot. Individual contracts, components, operations, flows, views, playbooks, and Store-X package manifests also carry versions and should be advanced when their public schema, ABI, behavior, or package contents change.

The supported public Rust crates are `industrialx-core`, `industrialx-timeseries`,
`industrialx-features`, and `industrialx-baseline-models`. See
[`docs/crate-releases.md`](docs/crate-releases.md) for exact dependency syntax,
MSRV, compatibility, ownership, and release-order details. Other helper and component
crates remain private workspace implementation details.
