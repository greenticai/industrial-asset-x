# industrialx-features

Deterministic, dependency-free feature engineering for Industrial-X consumers.

```rust
use industrialx_features::{feature_vector, residual};

assert_eq!(residual(12.0, 10.0), 2.0);
assert_eq!(feature_vector(&[1.0, 2.0, 4.0], 3), vec![4.0, 7.0 / 3.0, 2.0]);
```

This crate performs no I/O and requires no runtime capabilities. See the repository
versioning policy for compatibility and support expectations.
