#![doc = include_str!("../README.md")]

/// Named numeric feature with human-readable evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct NamedFeature {
    /// Stable feature name.
    pub name: String,
    /// Computed feature value.
    pub value: f64,
    /// Short explanation of the calculation.
    pub evidence: String,
}

/// Divides temperature by load, returning temperature unchanged at zero load.
pub fn load_normalized_temperature(temperature: f64, load: f64) -> f64 {
    if load.abs() < f64::EPSILON {
        return temperature;
    }
    temperature / load
}

/// Returns `actual - expected`.
pub fn residual(actual: f64, expected: f64) -> f64 {
    actual - expected
}

/// Returns latest value, latest rolling mean, and latest first difference.
pub fn feature_vector(values: &[f64], window: usize) -> Vec<f64> {
    let latest = values.last().copied().unwrap_or_default();
    let latest_mean = if window == 0 || values.is_empty() {
        0.0
    } else {
        let start = values.len().saturating_sub(window);
        let trailing = &values[start..];
        trailing.iter().sum::<f64>() / trailing.len() as f64
    };
    let latest_rate = values
        .windows(2)
        .last()
        .map_or(0.0, |pair| pair[1] - pair[0]);
    vec![latest, latest_mean, latest_rate]
}

/// Returns [`feature_vector`] values with stable names and evidence.
pub fn named_feature_vector(values: &[f64], window: usize) -> Vec<NamedFeature> {
    let vector = feature_vector(values, window);
    [
        ("latest_value", "latest observed value"),
        ("rolling_mean", "rolling mean over configured window"),
        ("rate_of_change", "latest first difference"),
    ]
    .iter()
    .zip(vector)
    .map(|((name, evidence), value)| NamedFeature {
        name: (*name).to_string(),
        value,
        evidence: (*evidence).to_string(),
    })
    .collect()
}

/// Serializes numeric features as a stable comma-separated string.
pub fn serialize_feature_vector(features: &[f64]) -> String {
    features
        .iter()
        .map(|feature| format!("{feature:.6}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// Serializes named features as stable semicolon-separated `name=value` pairs.
pub fn serialize_named_features(features: &[NamedFeature]) -> String {
    features
        .iter()
        .map(|feature| format!("{}={:.6}", feature.name, feature.value))
        .collect::<Vec<_>>()
        .join(";")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_features() {
        assert_eq!(serialize_feature_vector(&[1.0, 2.5]), "1.000000,2.500000");
    }

    #[test]
    fn serializes_named_features() {
        let features = named_feature_vector(&[1.0, 2.0, 4.0], 3);
        assert_eq!(features.len(), 3);
        assert!(serialize_named_features(&features).contains("latest_value=4.000000"));
    }
}
