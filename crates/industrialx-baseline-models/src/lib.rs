#![doc = include_str!("../README.md")]

use std::fmt;

/// Inclusive normal operating band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlBand {
    /// Inclusive lower bound.
    pub lower: f64,
    /// Inclusive upper bound.
    pub upper: f64,
}

/// Coarse risk classification derived from a health score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Health score from 80 through 100.
    Low,
    /// Health score from 50 through 79.
    Medium,
    /// Health score below 50.
    High,
}

/// Control-band health score and its supporting evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthAssessment {
    /// Bounded health score from 0 through 100.
    pub health_score: u8,
    /// Bounded anomaly score from 0.0 through 1.0.
    pub anomaly_score: f64,
    /// Signed distance outside the control band, or zero inside it.
    pub residual: f64,
    /// Risk classification derived from `health_score`.
    pub risk_level: RiskLevel,
}

/// A deterministic, load-aware equipment temperature baseline.
///
/// The load contribution is deliberately quadratic to approximate load-related
/// losses. Cooling activity is the fraction of configured cooling stages
/// reported as commanded or active; it is not a measurement of cooling
/// effectiveness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalBaseline {
    /// Expected rise above ambient at zero load.
    pub no_load_rise_c: f64,
    /// Temperature-rise coefficient applied to squared normalized load.
    pub load_rise_c: f64,
    /// Maximum relief associated with reported cooling activity.
    pub cooling_relief_c: f64,
    /// Half-width of the expected-temperature control band.
    pub residual_band_c: f64,
}

/// Expected-temperature evidence and the established control-band assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalAssessment {
    /// Model-derived expected temperature.
    pub expected_temperature_c: f64,
    /// Observed temperature minus expected temperature.
    pub thermal_residual_c: f64,
    /// Existing generic control-band assessment.
    pub health: HealthAssessment,
}

/// Error returned when a thermal baseline cannot produce a finite assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalBaselineError {
    /// A runtime input or model coefficient was NaN or infinite.
    NonFiniteInput { field: &'static str },
    /// Finite values overflowed during calculation.
    NonFiniteComputation { field: &'static str },
}

impl fmt::Display for ThermalBaselineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteInput { field } => {
                write!(formatter, "thermal baseline input `{field}` must be finite")
            }
            Self::NonFiniteComputation { field } => {
                write!(
                    formatter,
                    "thermal baseline computation `{field}` is not finite"
                )
            }
        }
    }
}

impl std::error::Error for ThermalBaselineError {}

impl ThermalBaseline {
    fn require_finite(value: f64, field: &'static str) -> Result<(), ThermalBaselineError> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(ThermalBaselineError::NonFiniteInput { field })
        }
    }

    fn validate(&self) -> Result<(), ThermalBaselineError> {
        Self::require_finite(self.no_load_rise_c, "no_load_rise_c")?;
        Self::require_finite(self.load_rise_c, "load_rise_c")?;
        Self::require_finite(self.cooling_relief_c, "cooling_relief_c")?;
        Self::require_finite(self.residual_band_c, "residual_band_c")
    }

    /// Calculates expected temperature from ambient temperature, normalized
    /// load, and reported cooling-stage activity.
    ///
    /// Finite fractions are clamped to `0.0..=1.0`. Non-finite inputs and
    /// coefficients return an error. The load term is
    /// `load_rise_c * load_fraction.powi(2)`.
    pub fn expected_temperature(
        &self,
        ambient_temperature_c: f64,
        load_fraction: f64,
        cooling_activity_fraction: f64,
    ) -> Result<f64, ThermalBaselineError> {
        self.validate()?;
        Self::require_finite(ambient_temperature_c, "ambient_temperature_c")?;
        Self::require_finite(load_fraction, "load_fraction")?;
        Self::require_finite(cooling_activity_fraction, "cooling_activity_fraction")?;

        let load = load_fraction.clamp(0.0, 1.0);
        let cooling_activity = cooling_activity_fraction.clamp(0.0, 1.0);
        let expected =
            ambient_temperature_c + self.no_load_rise_c + self.load_rise_c * load.powi(2)
                - self.cooling_relief_c * cooling_activity;
        if expected.is_finite() {
            Ok(expected)
        } else {
            Err(ThermalBaselineError::NonFiniteComputation {
                field: "expected_temperature_c",
            })
        }
    }

    /// Assesses observed temperature against the load-aware expected value.
    ///
    /// Cooling activity describes reported commanded or active stages rather
    /// than measured effectiveness. Elevated temperature despite high reported
    /// activity can therefore provide evidence of cooling underperformance.
    pub fn assess(
        &self,
        observed_temperature_c: f64,
        ambient_temperature_c: f64,
        load_fraction: f64,
        cooling_activity_fraction: f64,
    ) -> Result<ThermalAssessment, ThermalBaselineError> {
        Self::require_finite(observed_temperature_c, "observed_temperature_c")?;
        let expected = self.expected_temperature(
            ambient_temperature_c,
            load_fraction,
            cooling_activity_fraction,
        )?;
        let band_width = self.residual_band_c.abs();
        let lower = expected - band_width;
        let upper = expected + band_width;
        if !lower.is_finite() || !upper.is_finite() {
            return Err(ThermalBaselineError::NonFiniteComputation {
                field: "control_band",
            });
        }
        let thermal_residual_c = observed_temperature_c - expected;
        if !thermal_residual_c.is_finite() {
            return Err(ThermalBaselineError::NonFiniteComputation {
                field: "thermal_residual_c",
            });
        }
        let band = ControlBand { lower, upper };
        Ok(ThermalAssessment {
            expected_temperature_c: expected,
            thermal_residual_c,
            health: assess_health(observed_temperature_c, band),
        })
    }
}

impl ControlBand {
    /// Returns signed distance outside the band, or zero inside it.
    pub fn residual(&self, value: f64) -> f64 {
        if value < self.lower {
            value - self.lower
        } else if value > self.upper {
            value - self.upper
        } else {
            0.0
        }
    }
}

/// Converts distance outside a control band into a bounded anomaly score.
pub fn anomaly_score(value: f64, band: ControlBand) -> f64 {
    let residual = band.residual(value).abs();
    if residual == 0.0 {
        0.0
    } else {
        (residual / (band.upper - band.lower).abs().max(1.0)).min(1.0)
    }
}

/// Converts a bounded anomaly score into a health score from 0 through 100.
pub fn health_score(anomaly_score: f64) -> u8 {
    let bounded = anomaly_score.clamp(0.0, 1.0);
    ((1.0 - bounded) * 100.0).round() as u8
}

/// Classifies a health score as low, medium, or high risk.
pub fn risk_level(score: u8) -> RiskLevel {
    match score {
        80..=100 => RiskLevel::Low,
        50..=79 => RiskLevel::Medium,
        _ => RiskLevel::High,
    }
}

/// Assesses a value using the established control-band scoring algorithm.
pub fn assess_health(value: f64, band: ControlBand) -> HealthAssessment {
    let residual = band.residual(value);
    let anomaly_score = anomaly_score(value, band);
    let health_score = health_score(anomaly_score);
    HealthAssessment {
        health_score,
        anomaly_score,
        residual,
        risk_level: risk_level(health_score),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_health_from_anomaly() {
        assert_eq!(health_score(0.25), 75);
    }

    #[test]
    fn assesses_health_with_evidence() {
        let assessment = assess_health(
            65.0,
            ControlBand {
                lower: 40.0,
                upper: 60.0,
            },
        );
        assert_eq!(assessment.health_score, 75);
        assert_eq!(assessment.risk_level, RiskLevel::Medium);
        assert_eq!(assessment.residual, 5.0);
    }

    fn thermal_baseline() -> ThermalBaseline {
        ThermalBaseline {
            no_load_rise_c: 20.0,
            load_rise_c: 30.0,
            cooling_relief_c: 8.0,
            residual_band_c: 4.0,
        }
    }

    #[test]
    fn derives_expected_temperature_from_operating_conditions() {
        let expected = thermal_baseline()
            .expected_temperature(25.0, 0.8, 1.0)
            .unwrap();
        assert!((expected - 56.2).abs() < 0.000_001);
    }

    #[test]
    fn assesses_cooling_degradation_with_evidence() {
        let assessment = thermal_baseline().assess(65.0, 25.0, 0.8, 1.0).unwrap();
        assert!((assessment.expected_temperature_c - 56.2).abs() < 0.000_001);
        assert!((assessment.thermal_residual_c - 8.8).abs() < 0.000_001);
        assert_eq!(assessment.health.risk_level, RiskLevel::High);
    }

    #[test]
    fn keeps_normal_operation_low_risk() {
        let assessment = thermal_baseline().assess(58.0, 25.0, 0.8, 1.0).unwrap();
        assert_eq!(assessment.health.health_score, 100);
        assert_eq!(assessment.health.risk_level, RiskLevel::Low);
    }

    #[test]
    fn clamps_finite_normalized_inputs() {
        let baseline = thermal_baseline();
        assert_eq!(
            baseline.expected_temperature(25.0, 2.0, -1.0),
            baseline.expected_temperature(25.0, 1.0, 0.0)
        );
    }

    #[test]
    fn rejects_non_finite_normalized_inputs() {
        let baseline = thermal_baseline();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                baseline.expected_temperature(25.0, value, 0.5),
                Err(ThermalBaselineError::NonFiniteInput {
                    field: "load_fraction"
                })
            );
            assert_eq!(
                baseline.expected_temperature(25.0, 0.5, value),
                Err(ThermalBaselineError::NonFiniteInput {
                    field: "cooling_activity_fraction"
                })
            );
        }
    }

    #[test]
    fn rejects_non_finite_temperatures_and_coefficients() {
        assert_eq!(
            thermal_baseline().expected_temperature(f64::NAN, 0.5, 0.5),
            Err(ThermalBaselineError::NonFiniteInput {
                field: "ambient_temperature_c"
            })
        );
        assert_eq!(
            thermal_baseline().assess(f64::INFINITY, 25.0, 0.5, 0.5),
            Err(ThermalBaselineError::NonFiniteInput {
                field: "observed_temperature_c"
            })
        );

        for (field, baseline) in [
            (
                "no_load_rise_c",
                ThermalBaseline {
                    no_load_rise_c: f64::NAN,
                    ..thermal_baseline()
                },
            ),
            (
                "load_rise_c",
                ThermalBaseline {
                    load_rise_c: f64::INFINITY,
                    ..thermal_baseline()
                },
            ),
            (
                "cooling_relief_c",
                ThermalBaseline {
                    cooling_relief_c: f64::NEG_INFINITY,
                    ..thermal_baseline()
                },
            ),
            (
                "residual_band_c",
                ThermalBaseline {
                    residual_band_c: f64::NAN,
                    ..thermal_baseline()
                },
            ),
        ] {
            assert_eq!(
                baseline.expected_temperature(25.0, 0.5, 0.5),
                Err(ThermalBaselineError::NonFiniteInput { field })
            );
        }
    }

    #[test]
    fn rejects_non_finite_computed_values_before_health_scoring() {
        let baseline = ThermalBaseline {
            no_load_rise_c: f64::MAX,
            load_rise_c: f64::MAX,
            cooling_relief_c: 0.0,
            residual_band_c: 4.0,
        };
        assert_eq!(
            baseline.expected_temperature(f64::MAX, 1.0, 0.0),
            Err(ThermalBaselineError::NonFiniteComputation {
                field: "expected_temperature_c"
            })
        );
    }
}
