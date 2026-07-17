#![doc = include_str!("../README.md")]

/// Computes a trailing rolling mean with partial windows at the beginning.
pub fn rolling_mean(values: &[f64], window: usize) -> Vec<f64> {
    if window == 0 || values.is_empty() {
        return Vec::new();
    }
    (0..values.len())
        .map(|index| {
            let start = index.saturating_add(1).saturating_sub(window);
            let slice = &values[start..=index];
            slice.iter().sum::<f64>() / slice.len() as f64
        })
        .collect()
}

/// Computes means over a trailing timestamp window measured in milliseconds.
pub fn rolling_mean_by_time(points: &[(i64, f64)], window_ms: i64) -> Vec<f64> {
    if window_ms <= 0 {
        return Vec::new();
    }
    points
        .iter()
        .enumerate()
        .map(|(index, (timestamp, _))| {
            let min_ts = timestamp - window_ms;
            let start = points[..=index]
                .iter()
                .position(|(candidate_ts, _)| *candidate_ts >= min_ts)
                .unwrap_or(index);
            let slice = &points[start..=index];
            slice.iter().map(|(_, value)| *value).sum::<f64>() / slice.len() as f64
        })
        .collect()
}

/// Computes population standard deviation over trailing windows.
pub fn rolling_stddev(values: &[f64], window: usize) -> Vec<f64> {
    let means = rolling_mean(values, window);
    means
        .iter()
        .enumerate()
        .map(|(index, mean)| {
            let start = index.saturating_add(1).saturating_sub(window);
            let slice = &values[start..=index];
            let variance = slice
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / slice.len() as f64;
            variance.sqrt()
        })
        .collect()
}

/// Computes first differences between adjacent values.
pub fn rate_of_change(values: &[f64]) -> Vec<f64> {
    values.windows(2).map(|pair| pair[1] - pair[0]).collect()
}

/// Computes value change per millisecond between timestamped points.
pub fn rate_of_change_by_time(points: &[(i64, f64)]) -> Vec<f64> {
    points
        .windows(2)
        .map(|pair| {
            let dt = (pair[1].0 - pair[0].0) as f64;
            if dt == 0.0 {
                0.0
            } else {
                (pair[1].1 - pair[0].1) / dt
            }
        })
        .collect()
}

/// Returns inclusive index ranges containing equal values for at least `min_run` samples.
pub fn flatline_ranges(values: &[f64], min_run: usize) -> Vec<(usize, usize)> {
    if min_run < 2 || values.len() < min_run {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    for index in 1..values.len() {
        if values[index] != values[start] {
            if index - start >= min_run {
                ranges.push((start, index - 1));
            }
            start = index;
        }
    }
    if values.len() - start >= min_run {
        ranges.push((start, values.len() - 1));
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_rolling_mean() {
        assert_eq!(rolling_mean(&[1.0, 2.0, 3.0], 2), vec![1.0, 1.5, 2.5]);
    }

    #[test]
    fn detects_flatlines() {
        assert_eq!(
            flatline_ranges(&[1.0, 1.0, 2.0, 3.0, 3.0], 2),
            vec![(0, 1), (3, 4)]
        );
    }

    #[test]
    fn computes_time_window_mean() {
        assert_eq!(
            rolling_mean_by_time(&[(0, 1.0), (10, 3.0), (30, 9.0)], 15),
            vec![1.0, 2.0, 9.0]
        );
    }

    #[test]
    fn handles_invalid_windows() {
        assert!(rolling_mean(&[1.0, 2.0], 0).is_empty());
        assert!(rolling_mean_by_time(&[(0, 1.0)], 0).is_empty());
        assert!(flatline_ranges(&[1.0], 2).is_empty());
    }

    #[test]
    fn computes_stddev_and_rates() {
        let stddev = rolling_stddev(&[1.0, 3.0, 5.0], 3);
        assert_eq!(stddev[0], 0.0);
        assert!((stddev[2] - 1.632993161855452).abs() < 0.000001);
        assert_eq!(rate_of_change(&[1.0, 4.0, 9.0]), vec![3.0, 5.0]);
    }

    #[test]
    fn computes_time_based_rates() {
        assert_eq!(
            rate_of_change_by_time(&[(0, 1.0), (10, 6.0), (10, 9.0)]),
            vec![0.5, 0.0]
        );
    }
}
