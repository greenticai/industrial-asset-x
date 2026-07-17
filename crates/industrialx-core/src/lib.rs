#![doc = include_str!("../README.md")]

/// Quality state associated with a telemetry sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// A directly measured, usable value.
    Good,
    /// No value was available.
    Missing,
    /// A value was received but marked unusable.
    Bad,
    /// A value was estimated rather than directly measured.
    Estimated,
}

/// Timestamped telemetry value and its quality state.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Unix timestamp in milliseconds.
    pub timestamp_ms: i64,
    /// Numeric value, or `None` when absent.
    pub value: Option<f64>,
    /// Quality state supplied with the sample.
    pub quality: Quality,
}

/// Inclusive numeric range used for deterministic validation.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeCheck {
    /// Inclusive lower bound.
    pub min: f64,
    /// Inclusive upper bound.
    pub max: f64,
}

impl RangeCheck {
    /// Returns whether `value` lies within the inclusive range.
    pub fn contains(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}

/// Returns the fraction of missing or explicitly missing samples.
pub fn missing_ratio(samples: &[Sample]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let missing = samples
        .iter()
        .filter(|sample| sample.value.is_none() || sample.quality == Quality::Missing)
        .count();
    missing as f64 / samples.len() as f64
}

/// Returns the fraction of samples containing a good-quality value.
pub fn coverage_ratio(samples: &[Sample]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let usable = samples
        .iter()
        .filter(|sample| sample.value.is_some() && sample.quality == Quality::Good)
        .count();
    usable as f64 / samples.len() as f64
}

/// Returns indices of values outside `check`.
pub fn range_violations(values: &[f64], check: RangeCheck) -> Vec<usize> {
    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (!check.contains(*value)).then_some(index))
        .collect()
}

/// Sorts owned samples by ascending timestamp.
pub fn sorted_samples(mut samples: Vec<Sample>) -> Vec<Sample> {
    samples.sort_by_key(|sample| sample.timestamp_ms);
    samples
}

/// Rejects adjacent duplicate timestamps in an already ordered sample slice.
///
/// The duplicate timestamp is returned as the error.
pub fn reject_duplicate_timestamps(samples: &[Sample]) -> Result<(), i64> {
    for pair in samples.windows(2) {
        if pair[0].timestamp_ms == pair[1].timestamp_ms {
            return Err(pair[0].timestamp_ms);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing_ratio() {
        let samples = [
            Sample {
                timestamp_ms: 1,
                value: Some(1.0),
                quality: Quality::Good,
            },
            Sample {
                timestamp_ms: 2,
                value: None,
                quality: Quality::Missing,
            },
        ];
        assert_eq!(missing_ratio(&samples), 0.5);
    }

    #[test]
    fn detects_range_violations() {
        assert_eq!(
            range_violations(&[1.0, 5.0, 10.0], RangeCheck { min: 2.0, max: 8.0 }),
            vec![0, 2]
        );
    }

    #[test]
    fn computes_coverage_ratio() {
        let samples = [
            Sample {
                timestamp_ms: 1,
                value: Some(1.0),
                quality: Quality::Good,
            },
            Sample {
                timestamp_ms: 2,
                value: Some(2.0),
                quality: Quality::Bad,
            },
        ];
        assert_eq!(coverage_ratio(&samples), 0.5);
    }
}
