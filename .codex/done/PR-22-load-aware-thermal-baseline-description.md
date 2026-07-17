# Add load-aware thermal baseline model

## What changed

- Add a customer-neutral `ThermalBaseline` model to `industrialx-baseline-models`.
- Derive expected temperature from ambient temperature, a deliberately quadratic
  normalized-load term, reported cooling-stage activity, and configurable coefficients.
- Clamp finite normalized fractions and reject all non-finite runtime inputs and
  model coefficients before health scoring.
- Return field-specific `ThermalBaselineError` values when inputs or computed
  values are non-finite.
- Return the existing `HealthAssessment` together with expected temperature and
  thermal residual evidence.
- Cover normal operation, cooling degradation, load clamping, non-finite inputs,
  invalid coefficients, and arithmetic overflow with unit tests.

## Why

Industrial demos currently have to supply an already-computed expected temperature.
That keeps generic thermal-model behavior in demo fixtures instead of the reusable
Industrial-X engine. This API establishes the missing generic boundary without
introducing customer names, private sensor tags, presentation content, or branding.

## Impact

Consumers such as `koncar-demo` can provide their own mapped telemetry and model
coefficients, then reuse Industrial-X for both expected-temperature calculation and
health assessment. The new calculations return `Result` so invalid telemetry cannot
silently enter health scoring. Existing APIs are unchanged.

Cooling activity represents the fraction of configured cooling stages reported as
commanded or active, not measured cooling effectiveness. An elevated temperature
despite reported activity can therefore indicate cooling underperformance.

## Validation

```text
cargo test -p industrialx-baseline-models
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
