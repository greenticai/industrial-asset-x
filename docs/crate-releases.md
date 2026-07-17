# Supported Rust crate releases

The supported public Rust surface consists of four crates at the same release
version:

- `industrialx-core` for samples, quality, coverage, and validation;
- `industrialx-timeseries` for rolling and time-based calculations;
- `industrialx-features` for feature and residual generation;
- `industrialx-baseline-models` for control-band and thermal assessment.

Downstream applications use crates.io dependencies and do not require this repository:

```toml
[dependencies]
industrialx-core = "0.1.0"
industrialx-timeseries = "0.1.0"
industrialx-features = "0.1.0"
industrialx-baseline-models = "0.1.0"
```

All four crates require Rust 1.95 or newer. They are deterministic libraries with no
network, filesystem, secret, or runtime-provider requirements.

## Compatibility and upgrades

The crates follow Semantic Versioning. Patch releases preserve public API and model
behavior. Minor releases may add APIs but preserve existing behavior. Removing or
renaming public items, changing error behavior, changing numeric model equations, or
changing serialized formats requires a major version. Consumers should commit
`Cargo.lock`, review changelogs, and test representative telemetry before upgrading.

The Greentic maintainers own these crates. Support and security reports use the
repository issue tracker. Each crate includes its own README and changelog, and the
repository release tag is the authoritative source for a coordinated version.

## Release order and partial releases

Publish in this order:

1. `industrialx-timeseries`
2. `industrialx-core`
3. `industrialx-baseline-models`
4. `industrialx-features`
5. `industrial-asset-x`

The public crates deliberately have no registry dependencies on one another, allowing
all artifacts to be verified before the first publication. The release workflow still
uses the stable order above and waits for each published crate to become resolvable
before continuing. If publication stops partway, already published immutable versions
are not overwritten; fix the failure and rerun the same tagged release, which skips
versions already present.
