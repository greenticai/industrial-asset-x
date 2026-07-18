# Supported Rust crate releases

The supported public Rust surface consists of four crates at the same release
version:

- `industrialx-core` for samples, quality, coverage, and validation;
- `industrialx-timeseries` for rolling and time-based calculations;
- `industrialx-features` for feature and residual generation;
- `industrialx-baseline-models` for control-band and thermal assessment.

The crates are distributed from coordinated GitHub Releases rather than crates.io.
Downstream Cargo projects can pin the release tag directly:

```toml
[dependencies]
industrialx-core = { git = "https://github.com/greentic-biz/industrial-asset-x", tag = "v0.1.0" }
industrialx-timeseries = { git = "https://github.com/greentic-biz/industrial-asset-x", tag = "v0.1.0" }
industrialx-features = { git = "https://github.com/greentic-biz/industrial-asset-x", tag = "v0.1.0" }
industrialx-baseline-models = { git = "https://github.com/greentic-biz/industrial-asset-x", tag = "v0.1.0" }
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

## Release contents

The release workflow packages in this stable order:

1. `industrialx-timeseries`
2. `industrialx-core`
3. `industrialx-baseline-models`
4. `industrialx-features`
5. `industrial-asset-x`

The libraries deliberately have no dependencies on one another. CI builds each
`.crate` archive, verifies its contents, creates SHA-256 checksums, and uploads
the immutable artifacts to the matching GitHub Release. Cargo metadata sets
`publish = false`, so neither CI nor a local `cargo publish` can upload these packages
to crates.io.
