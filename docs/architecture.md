# Architecture

`industrial-asset-x` is organized around three generic namespaces:

- `assetx.*` for asset, component, sensor, failure-mode, maintenance, and health-state resources.
- `industrialx.*` for deterministic analytics, data quality, features, predictions, recommendations, feedback, synthetic scenarios, and model evaluation.
- `storex.*` for reusable packages, publishers, licenses, components, playbook listings, validation reports, and compatibility reports.

Rust crates provide deterministic core logic. WASM component packages expose that logic to GX operations. Flows compose operations, while views define neutral rendering contracts for downstream applications.
