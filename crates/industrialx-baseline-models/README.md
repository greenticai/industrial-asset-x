# industrialx-baseline-models

Deterministic control-band scoring and load-aware thermal baselines for
Industrial-X asset health.

```rust
use industrialx_baseline_models::ThermalBaseline;

let model = ThermalBaseline {
    no_load_rise_c: 20.0,
    load_rise_c: 30.0,
    cooling_relief_c: 8.0,
    residual_band_c: 4.0,
};
let assessment = model.assess(58.0, 25.0, 0.8, 1.0)?;
assert_eq!(assessment.health.health_score, 100);
# Ok::<(), industrialx_baseline_models::ThermalBaselineError>(())
```

All inputs are deterministic and capability-free. Non-finite inputs return a
field-specific error before health scoring.
