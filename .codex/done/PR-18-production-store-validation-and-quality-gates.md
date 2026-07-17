# PR 18 — Replace shell-only Store-X checks with production validation

## Goal

Move Store-X validation from simple shell/grep checks to a reusable Rust validator with clear diagnostics and policy fixtures.

## Placeholder removal targets

```text
scripts/validate-store-package.sh grep-based field checks
tests/store_package_validation.rs limited hard-coded package checks
store/schemas/*.schema.json permissive schemas
docs/* quality gates without executable policy coverage
```

## Scope

- Add a Rust validation module for Store-X package manifests.
- Validate schemas, package references, component references, flow/view/playbook paths, fixtures, licensing, publisher metadata, validation reports, compatibility reports, and customer-leakage policy.
- Emit machine-readable diagnostics with stable codes.
- Keep the shell script as a thin wrapper around the Rust CLI.
- Add validation fixtures for passing and failing packages.

## Acceptance criteria

- Store validation is available through `industrial-asset-x store validate`.
- `ci/local_check.sh` fails on broken package references or banned content.
- Diagnostics include file path, field path, error code, and message.
- Tests cover missing fields, invalid references, banned terms, invalid license metadata, and missing fixtures.

## Out of scope

- Payment/licensing enforcement.
- Remote store publishing.
