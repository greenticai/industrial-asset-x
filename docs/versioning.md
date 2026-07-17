# Versioning

The root `industrial-asset-x` package version is the release version for the catalog snapshot.

Contract, component, operation, flow, view, playbook, and Store-X package manifests each carry their own semantic version. These versions must be advanced when their public schema, ABI, behavior, or package contents change.

`industrialx-core`, `industrialx-timeseries`, `industrialx-features`, and
`industrialx-baseline-models` are supported crates.io boundaries. They share the
repository release version and follow the compatibility policy in
[`crate-releases.md`](crate-releases.md). Other helper and component crates remain
`publish = false` until their release boundaries are complete.

Release order for future publishable crates should follow dependency direction:

1. `industrialx-timeseries`.
2. `industrialx-core`.
3. `industrialx-baseline-models`.
4. `industrialx-features`.
5. Root catalog package.
