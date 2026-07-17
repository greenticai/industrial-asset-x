# Repository Overview

## 1. High-Level Purpose

`industrial-asset-x` is a generic Rust/WASM-first Greentic catalog foundation for industrial asset health. It defines customer-neutral Asset-X, Industrial-X, and Store-X contracts, deterministic Rust analytics crates, reusable component packages, GX operation manifests, reusable flows/views, a transformer health playbook, store validation assets, and release automation.

The repository is a Rust workspace. The root crate is the publishable catalog package and CLI. Helper crates and component crates are workspace members with `publish = false` until their independent release boundaries are finalized.

## 2. Main Components and Functionality

- **Path:** `src/lib.rs`
  - **Role:** Root catalog validation library.
  - **Key functionality:** Provides catalog summaries, contract/component listing, structured catalog validation, Store-X package validation, stable JSON reports, duplicate-id validation, and deterministic file discovery.
  - **Key dependencies / integration points:** Used by the CLI, store validator shell wrapper, tests, benchmarks, and `ci/local_check.sh`.

- **Path:** `src/main.rs`
  - **Role:** Offline catalog CLI.
  - **Key functionality:** Supports `catalog summary`, `catalog validate`, `store validate <package-dir>`, `contracts list`, and `components list`.
  - **Key dependencies / integration points:** Replaces the previous placeholder binary output.

- **Path:** `Cargo.toml`
  - **Role:** Root package and workspace manifest.
  - **Key functionality:** Defines workspace members, root publish metadata, package
    include rules, the Rust 1.95 MSRV, shared package metadata, and the authoritative
    versions/paths for all external and internal workspace dependencies. Member crates
    and components inherit these declarations with `workspace = true`.

- **Path:** `catalog.json`
  - **Role:** Machine-readable catalog descriptor.
  - **Key functionality:** Declares `assetx`, `industrialx`, and `storex` namespaces, Rust/WASM-only intent, and customer-neutral content policy.

- **Path:** `contracts/assetx`, `contracts/industrialx`, `contracts/storex`
  - **Role:** Resource contracts.
  - **Key functionality:** Each `contract.json` contains manifest metadata, lifecycle states, mutation rules, closed JSON schema, and a synthetic example with resource-specific `spec.resource_type`.

- **Path:** `crates/industrialx-core`
  - **Role:** Core deterministic Industrial-X helpers.
  - **Key functionality:** Defines typed samples with quality flags, range checks, missing-data ratio, coverage ratio, deterministic ordering, and duplicate timestamp rejection. It is a supported crates.io boundary with MSRV, README, changelog, package-content controls, and public API documentation.

- **Path:** `crates/industrialx-timeseries`
  - **Role:** Time-series helper crate.
  - **Key functionality:** Provides rolling mean/stddev, timestamp-aware rolling mean, rate of change, timestamp-aware rate of change, and flatline detection. It is a supported dependency-free crates.io boundary.

- **Path:** `crates/industrialx-features`
  - **Role:** Feature engineering helper crate.
  - **Key functionality:** Provides load-normalized temperature, residuals, numeric feature vectors, named feature vectors with evidence, and stable serialization. It is a supported dependency-free crates.io boundary.

- **Path:** `crates/industrialx-baseline-models`
  - **Role:** Baseline scoring helper crate.
  - **Key functionality:** Provides control bands, residual-based anomaly scoring, health score conversion, risk levels, and health assessment output with evidence fields. Its load-aware thermal baseline derives expected temperature from ambient temperature, quadratic normalized load, and reported cooling-stage activity, with explicit rejection of non-finite inputs and computations. It is a supported crates.io boundary with documented error and model behavior.

- **Path:** `crates/industrialx-wasm-abi`
  - **Role:** ABI naming and payload wrapper crate.
  - **Key functionality:** Defines component input/output structs and world constants for feature extraction, health scoring, anomaly detection, synthetic generation, and model evaluation.

- **Path:** `crates/industrialx-synthetic`
  - **Role:** Rule-based synthetic scenario helper crate.
  - **Key functionality:** Generates deterministic labeled stress-test scenarios and values for temperature drift, cooling degradation, sensor flatline, current imbalance, and load-normalized overheating.

- **Path:** `components/`
  - **Role:** Reusable component packages.
  - **Key functionality:** Contains answer-document-generated Greentic 0.6 component crates for signal profiling, data quality, feature extraction, health scoring, synthetic generation, and consolidated Industrial-X analytics. Custom Rust is limited to deterministic domain operations layered onto wizard-generated ABI scaffolds.

- **Path:** `component_abi/`
  - **Role:** Component interface documentation.
  - **Key functionality:** Documents initial feature extractor, health scorer, anomaly detector, synthetic generator, and model evaluator ABI worlds.

- **Path:** `ops/industrialx/`
  - **Role:** GX operation manifest packages.
  - **Key functionality:** Defines operation manifests, closed input/output schemas, permissions metadata, supported contracts, failure modes, component references, output contracts, and examples.

- **Path:** `flows/`
  - **Role:** Reusable GX flow definitions.
  - **Key functionality:** Defines ordered step graphs for site/asset modelling, data validation, baseline building, shadow health checks, and feedback improvement with evidence propagation.

- **Path:** `packs/industrialx-operations`
  - **Role:** Replayable Greentic Industrial-X operation pack source.
  - **Key functionality:** Uses `build-answer.json` and `flow.answers.json` as authoritative inputs to generate the pack, component bindings, three executable flow entrypoints, resolution lock, and provenance-bearing `.gtpack`; validation invokes the built WASM operation.

- **Path:** `views/`
  - **Role:** Neutral view definitions.
  - **Key functionality:** Defines reusable view descriptors with sections, field bindings, actions, and filters.

- **Path:** `playbooks/substation-transformer-health-basic`
  - **Role:** Reusable transformer health playbook.
  - **Key functionality:** Defines asset/component templates, sensor requirements with units/frequency/coverage, feature definitions linked to Rust implementations, baseline scoring, feedback questions, and machine-readable recommendation rules.

- **Path:** `store/`
  - **Role:** Store-X schemas and package examples.
  - **Key functionality:** Contains closed Store-X schemas and a transformer health package example with manifest, publisher, license, validation report, compatibility report, quality gates, and multi-case synthetic fixtures.

- **Path:** `scripts/validate-store-package.sh`
  - **Role:** Shell wrapper for Store-X validation.
  - **Key functionality:** Delegates to `cargo run -- store validate <package-dir>` so shell and Rust validation share one implementation.

- **Path:** `tests/`
  - **Role:** Integration and validation tests.
  - **Key functionality:** Covers CLI success/failure, component fixtures, catalog validation, Store-X validation, and lightweight concurrent catalog validation.

- **Path:** `benches/perf.rs`
  - **Role:** Criterion benchmark harness.
  - **Key functionality:** Benchmarks catalog summary and validation paths rather than placeholder arithmetic.

- **Path:** `ci/local_check.sh`
  - **Role:** Single developer and CI validation entrypoint.
  - **Key functionality:** Runs workspace format, clippy, tests, Rust-backed store validation, WASM component builds, build, docs, a repository-independent standalone public-crate consumer, package verification, package listing, and crates.io dry-run publishing in stable order.

- **Path:** `scripts/build-wasm-components.sh`
  - **Role:** WASM component build validation command.
  - **Key functionality:** Requires `cargo-component` and `wasm32-wasip2`, then builds every reusable component package with `cargo component build --release --target wasm32-wasip2`.

- **Path:** `coverage-policy.json`
  - **Role:** Automated coverage policy.
  - **Key functionality:** Sets 60% global/default line coverage minimums, stricter file-specific targets for high-risk Rust modules, and excludes only thin entrypoints/bench harness code.

- **Path:** `.github/workflows/`
  - **Role:** GitHub CI and release automation.
  - **Key functionality:** `ci.yml` runs lint, tests, and package dry-run checks. `publish.yml` verifies release tags against `Cargo.toml`, runs local checks, publishes crates with retry, and creates a GitHub Release.

- **Path:** `.codex/done/`
  - **Role:** Completed roadmap notes.
  - **Key functionality:** Contains PR-01 through PR-21 after implementation.

## 3. Work In Progress, TODOs, and Stubs

- **Location:** workspace crate publishing
  - **Status:** supported boundary established; remaining crates intentionally deferred
  - **Short description:** Four analytics crates are independently packageable and documented in `docs/crate-releases.md`; synthetic, ABI, and component crates remain `publish = false`.

## 4. Broken, Failing, or Conflicting Areas

- **Location:** `greentic-pack 1.1.5` doctor descriptor lookup
  - **Evidence:** Answer-generated `component@0.6.0` artifacts export `greentic:component/node@0.6.0` and pass actual `greentic-component test` invocation, lint, resolve, and pack build. `greentic-pack doctor` nevertheless reports `PACK_LOCK_COMPONENT_DESCRIBE_FAILED`; the same finding reproduces against the checked-in `greentic-demo` Telco-X artifact.
  - **Likely cause / nature of issue:** Installed pack doctor expects the older separate descriptor instance while the current component wizard emits the canonical node interface. The pack build accepts only this exact single compatibility finding and fails on every other doctor error.

- **Location:** repository checks
  - **Evidence:** `bash ci/local_check.sh` passed with workspace-wide format, clippy, tests, Rust-backed store validation, WASM component builds, build, docs, package verification, package listing, and `cargo publish --dry-run`. `greentic-dev coverage` passed with 82.02% workspace line coverage.
  - **Likely cause / nature of issue:** No current build, test, package, or dry-run publish failures were found.

## 5. Notes for Future Work

- Reassess whether synthetic and ABI helper crates should become independently publishable when downstream demand requires them.
- Keep shared core types aligned with existing Greentic crates before adding new cross-repo abstractions.
- Keep `.codex/repo_overview.md` refreshed before and after future PR-style work.
