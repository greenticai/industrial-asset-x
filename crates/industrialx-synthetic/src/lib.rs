#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioKind {
    GradualTemperatureDrift,
    CoolingFanDegradation,
    SensorFlatline,
    CurrentImbalanceTrend,
    LoadNormalizedOverheating,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntheticScenario {
    pub kind: ScenarioKind,
    pub label: &'static str,
    pub expected_failure_mode: &'static str,
    pub values: Vec<f64>,
}

pub fn generate_series(kind: ScenarioKind, length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| match kind {
            ScenarioKind::GradualTemperatureDrift => 40.0 + index as f64 * 0.1,
            ScenarioKind::CoolingFanDegradation => 1.0 - index as f64 / length.max(1) as f64,
            ScenarioKind::SensorFlatline => 42.0,
            ScenarioKind::CurrentImbalanceTrend => (index as f64).sin().abs() + index as f64 * 0.01,
            ScenarioKind::LoadNormalizedOverheating => 55.0 + (index as f64 * 0.2),
        })
        .collect()
}

pub fn generate_scenario(kind: ScenarioKind, length: usize) -> SyntheticScenario {
    let expected_failure_mode = match kind {
        ScenarioKind::GradualTemperatureDrift => "abnormal winding temperature",
        ScenarioKind::CoolingFanDegradation => "cooling system degradation",
        ScenarioKind::SensorFlatline => "sensor quality issue",
        ScenarioKind::CurrentImbalanceTrend => "current imbalance trend",
        ScenarioKind::LoadNormalizedOverheating => "load-related overheating",
    };
    SyntheticScenario {
        kind,
        label: "synthetic stress test",
        expected_failure_mode,
        values: generate_series(kind, length),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_requested_length() {
        assert_eq!(generate_series(ScenarioKind::SensorFlatline, 5).len(), 5);
    }

    #[test]
    fn generates_labeled_scenario() {
        let scenario = generate_scenario(ScenarioKind::CoolingFanDegradation, 4);
        assert_eq!(scenario.expected_failure_mode, "cooling system degradation");
        assert_eq!(scenario.values.len(), 4);
    }
}
