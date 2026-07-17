# PR 8 — Add generic flows and neutral views

## Goal

Compose Asset-X and Industrial-X operations into reusable GX flows and neutral views.

## Scope

Add MVP flows and reusable views for a generic industrial asset-health process.

## Adds flows

```text
flows/design-site-asset-model/
flows/validate-industrial-data/
flows/build-asset-health-baseline/
flows/run-shadow-health-check/
flows/record-feedback-and-improve/
```

## Adds views

```text
views/assetx/asset-hierarchy-review/
views/assetx/sensor-map-review/
views/assetx/asset-health-summary/
views/industrialx/data-quality-report/
views/industrialx/normal-behaviour-report/
views/industrialx/prediction-explanation/
views/industrialx/feedback-review/
```

## Flow outline

```text
design-site-asset-model
  -> validate-industrial-data
  -> build-asset-health-baseline
  -> run-shadow-health-check
  -> record-feedback-and-improve
```

## Acceptance criteria

- Flows use GX-native steps only.
- Views are neutral, not Webchat-specific.
- No customer-specific data or naming.
- Flows can be reused by demo packs.

## Out of scope

- Adaptive card rendering.
- Koncar demo wrappers.
