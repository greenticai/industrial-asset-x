# Codex task: add a load-aware thermal baseline

## Objective

Extend `industrial-asset-x` with a customer-neutral, load-aware thermal baseline so
demo repositories can reuse Industrial-X analytics instead of implementing thermal
model logic locally.

An implementation starting patch is provided in
`.codex/done/0001-add-load-aware-thermal-baseline.patch`. It demonstrates the model shape
and applies cleanly, but it predates the clarified non-finite-input requirements
below. Review and refine it rather than treating it as the final accepted diff.

## Context

`koncar-demo` already reuses these crates from this repository:

- `industrialx-core` for telemetry quality and coverage;
- `industrialx-features` for normalized temperature and residual evidence;
- `industrialx-baseline-models` for control-band health assessment.

The missing reusable primitive is calculation of expected equipment temperature
from ambient temperature, normalized load, and reported cooling activity. The demo
must not own that generic algorithm.

## Model semantics

`cooling_activity_fraction` means the normalized fraction of configured cooling
stages reported as commanded or active: `0.0` means none and `1.0` means all. It is
not a direct measurement of cooling effectiveness. Therefore, an elevated observed
temperature while cooling activity is reported as `1.0` is valid evidence of
possible cooling underperformance or degradation.

The load contribution is intentionally quadratic for this initial model:
`load_rise_c * load_fraction.powi(2)`. This is a deliberate, documented
physics-inspired approximation of load-related losses, not an accidental patch
author assumption and not a claim to implement a full transformer thermal standard.
Changing that equation later is a public model-behavior change and requires tests
and versioning review.

## Requested change

Update `crates/industrialx-baseline-models/src/lib.rs` with:

1. A configurable `ThermalBaseline` containing:
   - no-load temperature rise;
   - load-dependent temperature rise;
   - cooling relief from reported cooling activity;
   - residual control-band width.
2. An `expected_temperature` method using the explicit quadratic load equation and
   finite normalized load and cooling-activity inputs.
3. An `assess` method returning:
   - expected temperature;
   - observed-versus-expected thermal residual;
   - the existing `HealthAssessment` calculated with the existing control-band API.
4. Unit tests for:
   - expected-temperature calculation;
   - a cooling-degradation/anomaly case;
   - normal low-risk operation;
   - clamping finite normalized inputs outside `0.0..=1.0`;
   - rejecting NaN and positive/negative infinity;
   - rejecting non-finite model coefficients and observed/ambient temperatures.

## Constraints

- Keep the API generic and customer-neutral.
- Do not add Koncar names, branded sensor tags, adaptive cards, or demo fixture data.
- Reuse `ControlBand` and `assess_health`; do not create a parallel health-scoring
  implementation.
- Return an explicit error for every non-finite input. Do not pass NaN or infinity
  into `ControlBand` or health scoring, and do not silently coerce non-finite values.
- Finite normalized load and cooling activity outside `0.0..=1.0` may be clamped.
- Preserve existing public APIs and tests.
- Keep the implementation deterministic and compatible with Rust/WASM consumers.
- Do not change unrelated product files. Refresh `.codex/repo_overview.md` as
  required by the repository-wide workflow; that metadata update is in scope.

## Suggested workflow

From the repository root:

```sh
patch -p1 < .codex/0001-add-load-aware-thermal-baseline.patch
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p industrialx-baseline-models
cargo test --workspace
```

Inspect `.codex/done/PR-22-load-aware-thermal-baseline-description.md` for the proposed pull-request description. If the patch needs
refinement, update the implementation and PR description so they remain aligned.

## Acceptance criteria

- Consumers can calculate expected temperature without customer-specific logic.
- Consumers receive both thermal residual evidence and the established generic
  health assessment.
- Finite normalized inputs outside `0.0..=1.0` are clamped deterministically.
- NaN and infinity in runtime inputs or model coefficients return an explicit error
  before health scoring.
- Cooling activity semantics and the quadratic equation are documented in public
  API documentation.
- Existing and new tests pass.
- Formatting and clippy checks pass.
- No customer-specific content enters `industrial-asset-x`.
- `.codex/repo_overview.md` is refreshed according to the global repository rule.

## Downstream follow-up

After this upstream API is available, update `koncar-demo` to replace its embedded
expected-temperature fixture input with `ThermalBaseline::assess`. Keep presentation,
branding, and synthetic Koncar fixtures in the demo repository.
