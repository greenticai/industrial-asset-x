# PR 14 — Replace permissive contract scaffolds with strict schemas

## Goal

Turn broad `contract.json` scaffolds into production-ready, contract-specific schemas and examples for Asset-X, Industrial-X, and Store-X resources.

## Placeholder removal targets

```text
contracts/**/contract.json schema.additionalProperties = true
contracts/**/contract.json generic spec/evidence shapes
generic synthetic examples that do not exercise resource-specific fields
```

## Scope

- Split each contract package into clear files where useful:

```text
manifest.json
schema.json
example.json
lifecycle.json
mutation-rules.md
README.md
```

- Define strict required fields for each resource type.
- Add domain-specific validation constraints:
  - Asset/component hierarchy references.
  - Sensor units, sampling metadata, and physical quantity types.
  - Failure mode taxonomy and affected component links.
  - Prediction/recommendation evidence and confidence fields.
  - Store package licensing, publisher, validation, and compatibility metadata.

## Acceptance criteria

- Generic placeholder schema fields are replaced by resource-specific fields.
- `additionalProperties` is false unless there is a documented extension point.
- Every schema has at least one valid example and one invalid fixture.
- Rust tests validate all examples and reject invalid fixtures.
- No customer-specific names, tags, or data appear.

## Out of scope

- Runtime database storage.
- Customer-specific schema extensions.
