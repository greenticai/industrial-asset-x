# industrialx-timeseries

Deterministic rolling statistics, rates of change, and flatline detection for
Industrial-X analytics.

```rust
use industrialx_timeseries::rolling_mean;

assert_eq!(rolling_mean(&[1.0, 2.0, 3.0], 2), vec![1.0, 1.5, 2.5]);
```

This crate performs no I/O and requires no runtime capabilities. See the repository
versioning policy for compatibility and support expectations.
