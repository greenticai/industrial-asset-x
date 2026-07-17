# PR 1 — Bootstrap generic Rust/WASM industrial asset catalog

## Goal

Create the initial `industrial-asset-x` repository structure as a generic downstream Greentic-X catalog for industrial asset health.

## Scope

Add the top-level repo layout, documentation, naming rules, and placeholder directories for Asset-X, Industrial-X, Store-X, Rust crates, WASM components, playbooks, tests, and store metadata.

## Adds

```text
README.md
catalog.json

contracts/
  assetx/
  industrialx/
  storex/

ops/
  assetx/
  industrialx/
  storex/

resolvers/
  assetx/
  industrialx/
  storex/

flows/
views/
playbooks/
component_abi/
crates/
components/
store/
tests/
docs/
```

## Docs

```text
docs/architecture.md
docs/naming.md
docs/no-customer-specific-content.md
docs/rust-wasm-only.md
docs/store-model.md
docs/contribution-model.md
```

## Acceptance criteria

- Repo has a clear generic product/catalog structure.
- Naming rules explicitly ban customer-specific content.
- README explains relationship to Greentic-X, Asset-X, Industrial-X, Store-X, and demo repos.
- No Koncar-specific references.

## Out of scope

- Real contracts.
- Real WASM components.
- Demo content.
- Customer-specific material.
