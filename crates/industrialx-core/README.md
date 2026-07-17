# industrialx-core

Stable, deterministic telemetry quality primitives for Industrial-X consumers.

```rust
use industrialx_core::{coverage_ratio, Quality, Sample};

let coverage = coverage_ratio(&[Sample {
    timestamp_ms: 1,
    value: Some(42.0),
    quality: Quality::Good,
}]);
assert_eq!(coverage, 1.0);
```

This crate performs no I/O and requires no runtime capabilities. See the repository
versioning policy for compatibility and support expectations.
