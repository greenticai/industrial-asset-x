use industrialx_baseline_models::{ControlBand, ThermalBaseline, assess_health};
use serde_json::{Value, json};

fn failure(code: &str, message: impl Into<String>) -> Value {
    json!({"ok":false,"data":null,"evidence":[],"diagnostics":[{"code":code,"message":message.into()}]})
}

fn finite(input: &Value, field: &str) -> Result<f64, Value> {
    input
        .get(field)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| failure("invalid-input", format!("`{field}` must be finite")))
}

fn values(input: &Value) -> Result<Vec<f64>, Value> {
    input
        .get("values")
        .and_then(Value::as_array)
        .ok_or_else(|| failure("invalid-input", "`values` must be an array"))?
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value
                .as_f64()
                .filter(|value| value.is_finite())
                .ok_or_else(|| {
                    failure("invalid-input", format!("`values[{index}]` must be finite"))
                })
        })
        .collect()
}

/// Executes Industrial-X domain logic behind the generated Greentic component ABI.
pub fn run(operation: &str, input: &Value) -> Value {
    match operation {
        "profile-signals" => match values(input) {
            Ok(values) if !values.is_empty() => json!({
                "ok": true,
                "data": {
                    "sample_count": values.len(),
                    "mean": values.iter().sum::<f64>() / values.len() as f64,
                    "flatline_ranges": industrialx_timeseries::flatline_ranges(&values, 3)
                },
                "evidence": [{"kind":"signal-profile"}],
                "diagnostics": []
            }),
            Ok(_) => failure("invalid-input", "`values` must not be empty"),
            Err(error) => error,
        },
        "validate-data-quality" => {
            let missing = match finite(input, "missing_ratio") {
                Ok(value) => value,
                Err(error) => return error,
            };
            let coverage = match finite(input, "coverage_ratio") {
                Ok(value) => value,
                Err(error) => return error,
            };
            let usable = missing <= 0.05 && coverage >= 0.90;
            json!({
                "ok": usable,
                "data": {"missing_ratio":missing,"coverage_ratio":coverage,"usable_for_baseline":usable},
                "evidence": [{"kind":"quality-metrics"}],
                "diagnostics": if usable { vec![] } else { vec![json!({"code":"poor-data-quality","message":"quality thresholds failed"})] }
            })
        }
        "generate-features" => match values(input) {
            Ok(values) => {
                let window = input.get("window").and_then(Value::as_u64).unwrap_or(3) as usize;
                json!({"ok":true,"data":{"features":industrialx_features::feature_vector(&values,window)},"evidence":[{"kind":"feature-vector","window":window}],"diagnostics":[]})
            }
            Err(error) => error,
        },
        "assess-thermal-baseline" => {
            let names = [
                "observed_temperature_c",
                "ambient_temperature_c",
                "load_fraction",
                "cooling_activity_fraction",
                "no_load_rise_c",
                "load_rise_c",
                "cooling_relief_c",
                "residual_band_c",
            ];
            let mut data = Vec::new();
            for name in names {
                match finite(input, name) {
                    Ok(value) => data.push(value),
                    Err(error) => return error,
                }
            }
            let baseline = ThermalBaseline {
                no_load_rise_c: data[4],
                load_rise_c: data[5],
                cooling_relief_c: data[6],
                residual_band_c: data[7],
            };
            match baseline.assess(data[0], data[1], data[2], data[3]) {
                Ok(result) => json!({
                    "ok":true,
                    "data":{"expected_temperature_c":result.expected_temperature_c,"thermal_residual_c":result.thermal_residual_c,"health_score":result.health.health_score,"anomaly_score":result.health.anomaly_score,"risk":format!("{:?}",result.health.risk_level).to_lowercase()},
                    "evidence":[{"kind":"thermal-baseline","equation":"ambient + no_load_rise + load_rise * load^2 - cooling_relief * cooling_activity"}],
                    "diagnostics":[]
                }),
                Err(error) => failure("thermal-baseline-error", error.to_string()),
            }
        }
        "score-asset-health" => {
            let value = match finite(input, "value") {
                Ok(value) => value,
                Err(error) => return error,
            };
            let lower = match finite(input, "lower") {
                Ok(value) => value,
                Err(error) => return error,
            };
            let upper = match finite(input, "upper") {
                Ok(value) => value,
                Err(error) => return error,
            };
            if lower > upper {
                return failure("invalid-contract", "`lower` must not exceed `upper`");
            }
            let result = assess_health(value, ControlBand { lower, upper });
            json!({"ok":true,"data":{"health_score":result.health_score,"anomaly_score":result.anomaly_score,"residual":result.residual,"risk":format!("{:?}",result.risk_level).to_lowercase()},"evidence":[{"kind":"control-band","lower":lower,"upper":upper}],"diagnostics":[]})
        }
        _ => failure(
            "unknown-operation",
            format!("unsupported operation `{operation}`"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_signals_and_rejects_invalid_values() {
        let output = run("profile-signals", &json!({"values":[1.0,2.0,2.0,2.0]}));
        assert_eq!(output["ok"], true);
        assert_eq!(output["data"]["sample_count"], 4);
        assert_eq!(output["data"]["flatline_ranges"][0], json!([1, 3]));

        assert_eq!(run("profile-signals", &json!({"values":[]}))["ok"], false);
        assert_eq!(run("profile-signals", &json!({}))["ok"], false);
        assert_eq!(
            run("profile-signals", &json!({"values":["bad"]}))["ok"],
            false
        );
    }

    #[test]
    fn validates_good_and_poor_data_quality() {
        let good = run(
            "validate-data-quality",
            &json!({"missing_ratio":0.01,"coverage_ratio":0.99}),
        );
        assert_eq!(good["ok"], true);

        let poor = run(
            "validate-data-quality",
            &json!({"missing_ratio":0.20,"coverage_ratio":0.50}),
        );
        assert_eq!(poor["ok"], false);
        assert_eq!(poor["diagnostics"][0]["code"], "poor-data-quality");
        assert_eq!(
            run("validate-data-quality", &json!({"missing_ratio":0.0}))["ok"],
            false
        );
    }

    #[test]
    fn generates_features_with_default_and_explicit_windows() {
        let default = run("generate-features", &json!({"values":[1.0,2.0,4.0]}));
        assert_eq!(default["data"]["features"][0], 4.0);
        assert_eq!(default["evidence"][0]["window"], 3);

        let explicit = run(
            "generate-features",
            &json!({"values":[1.0,2.0,4.0],"window":2}),
        );
        assert_eq!(explicit["evidence"][0]["window"], 2);
        assert_eq!(run("generate-features", &json!({}))["ok"], false);
    }

    #[test]
    fn assesses_thermal_baseline_and_reports_bad_input() {
        let input = json!({
            "observed_temperature_c":58.0,
            "ambient_temperature_c":25.0,
            "load_fraction":0.8,
            "cooling_activity_fraction":1.0,
            "no_load_rise_c":20.0,
            "load_rise_c":30.0,
            "cooling_relief_c":8.0,
            "residual_band_c":4.0
        });
        let output = run("assess-thermal-baseline", &input);
        assert_eq!(output["ok"], true);
        assert_eq!(output["data"]["health_score"], 100);

        assert_eq!(
            run("assess-thermal-baseline", &json!({}))["diagnostics"][0]["code"],
            "invalid-input"
        );
    }

    #[test]
    fn scores_health_and_rejects_invalid_bands() {
        let output = run(
            "score-asset-health",
            &json!({"value":65.0,"lower":40.0,"upper":60.0}),
        );
        assert_eq!(output["ok"], true);
        assert_eq!(output["data"]["residual"], 5.0);

        let invalid = run(
            "score-asset-health",
            &json!({"value":50.0,"lower":60.0,"upper":40.0}),
        );
        assert_eq!(invalid["diagnostics"][0]["code"], "invalid-contract");
        assert_eq!(run("score-asset-health", &json!({}))["ok"], false);
    }

    #[test]
    fn rejects_unknown_operations() {
        let output = run("unknown", &json!({}));
        assert_eq!(output["diagnostics"][0]["code"], "unknown-operation");
    }
}
