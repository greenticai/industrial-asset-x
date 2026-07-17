# PR 4 — Add Store-X package and licensing contracts

## Goal

Enable reusable industrial capability packages to be validated, licensed, published, and reused across customers.

## Scope

Add `storex.*` contracts and manifest schemas for reusable package content.

## Adds contracts

```text
contracts/storex/package/
contracts/storex/publisher/
contracts/storex/license/
contracts/storex/component/
contracts/storex/playbook-listing/
contracts/storex/validation-report/
contracts/storex/compatibility-report/
```

## Adds schemas

```text
store/schemas/package-manifest.schema.json
store/schemas/license.schema.json
store/schemas/publisher.schema.json
store/schemas/component.schema.json
store/schemas/validation-report.schema.json
store/schemas/compatibility-report.schema.json
```

## Package types

```text
asset_template
sensor_mapping_template
failure_mode_library
maintenance_playbook
feature_extractor
normal_behaviour_model
health_scorer
anomaly_detector
synthetic_scenario_generator
model_evaluator
explanation_view
recommendation_flow
complete_solution_bundle
```

## Acceptance criteria

- A store package can declare supported asset types, required sensor types, included components, flows, views, playbooks, license, publisher, and validation report.
- Package manifest supports Rust/WASM components.
- Licensing metadata is explicit but not enforced in runtime yet.
- No customer-specific content.

## Out of scope

- Payment/licensing backend.
- Actual marketplace UI.
