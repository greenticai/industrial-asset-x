//! Feature resolution. A feature is `provided` (the caller precomputed it),
//! `computed` (a known implementation plus caller-bound sensors) or
//! `unresolved`. The playbook's definition gives an implementation name only,
//! so inputs are never guessed: the caller binds sensors explicitly.

use serde_json::{Map, Value, json};

use crate::execute::{Refusal, bad, finite_number};

const LOAD_NORMALIZED: &str = "industrialx-features::load_normalized_temperature";
const RATE_OF_CHANGE: &str = "industrialx-timeseries::rate_of_change";

pub struct FeatureResult {
    pub name: String,
    pub value: Option<f64>,
    pub source: &'static str,
    pub implementation: Option<String>,
    pub reason: Option<String>,
}

impl FeatureResult {
    pub fn to_json(&self) -> Value {
        let mut out = json!({
            "name": self.name,
            "status": if self.value.is_some() { self.source } else { "unresolved" },
            "implementation": self.implementation,
        });
        if let Some(value) = self.value {
            out["value"] = json!(value);
        }
        if let Some(reason) = &self.reason {
            out["reason"] = json!(reason);
        }
        out
    }

    fn unresolved(name: &str, implementation: Option<&str>, reason: impl Into<String>) -> Self {
        Self {
            name: name.to_owned(),
            value: None,
            source: "unresolved",
            implementation: implementation.map(str::to_owned),
            reason: Some(reason.into()),
        }
    }
}

fn implementation<'a>(playbook: &'a Value, name: &str) -> Option<&'a str> {
    playbook
        .get("feature_definitions")?
        .as_array()?
        .iter()
        .find(|d| d.get("name").and_then(Value::as_str) == Some(name))?
        .get("implementation")?
        .as_str()
}

fn scalar(readings: &Map<String, Value>, sensor: &str) -> Result<f64, String> {
    match readings.get(sensor) {
        None | Some(Value::Null) => Err(format!("reading `{sensor}` is missing")),
        Some(Value::Array(_)) => Err(format!(
            "reading `{sensor}` is a series and the playbook declares no reduction to a scalar"
        )),
        Some(value) => value
            .as_f64()
            .filter(|n| n.is_finite())
            .ok_or_else(|| format!("reading `{sensor}` is not a finite number")),
    }
}

fn inputs(binding: &Value, name: &str) -> Result<Vec<String>, Refusal> {
    let list = binding
        .get("inputs")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            bad(format!(
                "bindings.{name}.inputs must be an array of sensor names"
            ))
        })?;
    list.iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| bad(format!("bindings.{name}.inputs must be strings")))
        })
        .collect()
}

fn compute(
    name: &str,
    implementation: &str,
    sensors: &[String],
    readings: &Map<String, Value>,
) -> FeatureResult {
    let fail = |reason: String| FeatureResult::unresolved(name, Some(implementation), reason);
    match implementation {
        LOAD_NORMALIZED => {
            let [temperature, load] = sensors else {
                return fail("load_normalized_temperature needs exactly two inputs: [temperature, load]".into());
            };
            let (t, l) = match (scalar(readings, temperature), scalar(readings, load)) {
                (Ok(t), Ok(l)) => (t, l),
                (Err(reason), _) | (_, Err(reason)) => return fail(reason),
            };
            if l.abs() < f64::EPSILON {
                return fail("load is zero, so the ratio is undefined".into());
            }
            FeatureResult {
                name: name.to_owned(),
                value: Some(industrialx_features::load_normalized_temperature(t, l)),
                source: "computed",
                implementation: Some(implementation.to_owned()),
                reason: None,
            }
        }
        RATE_OF_CHANGE => fail(
            "rate_of_change returns a series and the playbook declares no reduction to a scalar; supply the feature precomputed".into(),
        ),
        other => fail(format!("no native binding for implementation `{other}`; supply the feature precomputed")),
    }
}

pub fn resolve(
    playbook: &Value,
    names: &[String],
    readings: &Map<String, Value>,
    provided: Option<&Map<String, Value>>,
    bindings: Option<&Map<String, Value>>,
) -> Result<Vec<FeatureResult>, Refusal> {
    let mut out = Vec::new();
    for name in names {
        let implementation = implementation(playbook, name);
        let given = provided.and_then(|p| p.get(name));
        let binding = bindings.and_then(|b| b.get(name));
        let result = match (given, binding) {
            (Some(_), Some(_)) => {
                return Err(bad(format!(
                    "feature `{name}` is both provided and bound; send one"
                )));
            }
            (Some(value), None) => FeatureResult {
                name: name.clone(),
                value: Some(finite_number(value, &format!("features.{name}"))?),
                source: "provided",
                implementation: implementation.map(str::to_owned),
                reason: None,
            },
            (None, Some(binding)) => match implementation {
                Some(implementation) => {
                    compute(name, implementation, &inputs(binding, name)?, readings)
                }
                None => FeatureResult::unresolved(
                    name,
                    None,
                    "the playbook has no definition for this feature",
                ),
            },
            (None, None) => FeatureResult::unresolved(
                name,
                implementation,
                "the definition names an implementation but not which sensors feed it; bind inputs or supply the feature precomputed",
            ),
        };
        out.push(result);
    }
    Ok(out)
}
