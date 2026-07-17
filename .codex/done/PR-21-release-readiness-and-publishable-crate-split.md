# PR 21 — Prepare production crate/package release boundaries

## Goal

Move from one publishable root crate plus unpublished workspace scaffolds to intentional release boundaries for production crates, components, schemas, and store packages.

## Placeholder removal targets

```text
workspace crates/components with publish = false by default
root package includes broad catalog assets
README release notes that do not describe component/store package versioning
```

## Scope

- Decide which crates should be published independently.
- Add complete crate metadata, README files, examples, and package include rules.
- Define versioning rules between Rust crates, component manifests, contract versions, and Store-X packages.
- Add package-content tests for each publishable crate.
- Update release workflow to publish crates in dependency order.

## Acceptance criteria

- Publishable crates have complete metadata and minimal package contents.
- Non-publishable artifacts have explicit rationale.
- Release workflow publishes in dependency order and still runs dry-run checks first.
- `cargo package --list` output is checked for required runtime assets.

## Out of scope

- GHCR publishing unless separately requested.
- cargo-binstall artifacts unless separately requested.
