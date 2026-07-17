# PR 10 — Add store validation and promotion checks

## Goal

Make sure reusable candidates can be validated before publication or licensing.

## Scope

Add validation scripts/tests/docs for store packages and promoted artifacts.

## Adds

```text
scripts/validate-store-package.sh
tests/store_package_validation.rs
docs/promotion-checklist.md
docs/store-quality-gates.md
docs/no-customer-leakage-checks.md
```

## Checks

```text
manifest complete
license metadata present
publisher metadata present
schemas valid
component tests pass
WASM artifact referenced
fixtures included
compatibility declared
no banned customer-specific terms
no raw customer data
```

## Acceptance criteria

- Store package validation can run locally and in CI.
- Promotion checklist is explicit.
- Validation supports reusable candidates coming from demo repos.

## Out of scope

- Publishing to store.
- Payment/licensing enforcement.
