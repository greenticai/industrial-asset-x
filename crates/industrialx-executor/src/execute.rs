//! `POST /execute`: deterministic, advisory-only evaluation of a playbook.
//!
//! The playbook is data. It names features by an `implementation` string but
//! not which sensors feed them, and it names a scoring method but not the
//! control band. Whatever the playbook plus the request do not fully determine
//! is returned as `unresolved` with the reason; no formula is invented.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::{rules, scoring, validate};

const ALLOWED_FIELDS: &[&str] = &[
    "playbook",
    "mode",
    "readings",
    "features",
    "bindings",
    "scoring",
    "failure_modes",
    "evidence",
];

/// Typed request-level failure, rendered as `{"error": {"code", "message"}}`.
#[derive(Debug)]
pub struct Refusal {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
    pub errors: Vec<Value>,
}

impl Refusal {
    pub fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            errors: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Value {
        let mut body = json!({"error": {"code": self.code, "message": self.message}});
        if !self.errors.is_empty() {
            body["errors"] = Value::Array(self.errors.clone());
        }
        body
    }
}

fn invalid(message: impl Into<String>) -> Refusal {
    Refusal::new(400, "invalid_request", message)
}

fn object<'a>(
    request: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a Map<String, Value>>, Refusal> {
    match request.get(key) {
        None => Ok(None),
        Some(Value::Object(map)) => Ok(Some(map)),
        Some(_) => Err(invalid(format!("`{key}` must be an object"))),
    }
}

fn names(playbook: &Value, key: &str, field: &str) -> Vec<String> {
    playbook
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| match field {
            "" => item.as_str(),
            name => item.get(name).and_then(Value::as_str),
        })
        .map(str::to_owned)
        .collect()
}

fn finite(value: &Value, what: &str) -> Result<f64, Refusal> {
    value
        .as_f64()
        .filter(|n| n.is_finite())
        .ok_or_else(|| invalid(format!("{what} must be a finite number")))
}

/// Present means non-null and not an empty collection.
fn present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Array(items) => !items.is_empty(),
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}

pub fn execute(request: &Value) -> Result<Value, Refusal> {
    let request = request
        .as_object()
        .ok_or_else(|| invalid("the request body must be a JSON object"))?;
    if let Some(unknown) = request
        .keys()
        .find(|k| !ALLOWED_FIELDS.contains(&k.as_str()))
    {
        return Err(Refusal::new(
            400,
            "unknown_field",
            format!("`{unknown}` is not a request field"),
        ));
    }
    let playbook = request
        .get("playbook")
        .ok_or_else(|| invalid("`playbook` is required"))?;
    let problems = validate::errors(playbook);
    if !problems.is_empty() {
        let mut refusal = Refusal::new(422, "playbook_invalid", "the playbook does not validate");
        refusal.errors = problems;
        return Err(refusal);
    }

    let readings = object(request, "readings")?.cloned().unwrap_or_default();
    let evidence = object(request, "evidence")?.cloned().unwrap_or_default();
    let feature_names = names(playbook, "features", "");

    // Request names must exist in the playbook; a typo is not silently ignored.
    for (field, map) in [
        ("features", object(request, "features")?),
        ("bindings", object(request, "bindings")?),
    ] {
        for key in map.into_iter().flat_map(Map::keys) {
            if !feature_names.contains(key) {
                return Err(Refusal::new(
                    422,
                    "unknown_feature",
                    format!("`{field}.{key}` names a feature the playbook does not declare"),
                ));
            }
        }
    }
    let modes = names(playbook, "failure_modes", "");
    let asserted = object(request, "failure_modes")?
        .cloned()
        .unwrap_or_default();
    if let Some(key) = asserted.keys().find(|k| !modes.contains(k)) {
        return Err(Refusal::new(
            422,
            "unknown_failure_mode",
            format!("`failure_modes.{key}` is not a failure mode of the playbook"),
        ));
    }

    let feature_results = crate::features::resolve(
        playbook,
        &feature_names,
        &readings,
        object(request, "features")?,
        object(request, "bindings")?,
    )?;
    let values: BTreeMap<String, f64> = feature_results
        .iter()
        .filter_map(|f| Some((f.name.clone(), f.value?)))
        .collect();

    let baseline = scoring::baseline(playbook, &values, request.get("scoring"))?;
    let failure_modes = rules::failure_modes(&modes, &asserted)?;
    let outcome = rules::evaluate(playbook, &baseline, &failure_modes, &evidence)?;

    let sensors: Vec<(String, bool)> = names(playbook, "sensor_requirements", "name")
        .into_iter()
        .zip(
            playbook
                .get("sensor_requirements")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|s| s.get("required").and_then(Value::as_bool).unwrap_or(false)),
        )
        .collect();
    let missing_sensors: Vec<&String> = sensors
        .iter()
        .filter(|(name, required)| *required && !readings.get(name).is_some_and(present))
        .map(|(name, _)| name)
        .collect();
    let declared: Vec<&String> = sensors.iter().map(|(name, _)| name).collect();

    let mut unresolved: Vec<Value> = Vec::new();
    for f in &feature_results {
        if let Some(reason) = &f.reason {
            unresolved.push(json!({"subject": "feature", "name": f.name, "reason": reason}));
        }
    }
    if let Some(reason) = baseline.get("reason") {
        unresolved.push(json!({"subject": "baseline_scoring", "name": "score", "reason": reason}));
    }
    for mode in &failure_modes {
        if let Some(reason) = &mode.reason {
            unresolved
                .push(json!({"subject": "failure_mode", "name": mode.name, "reason": reason}));
        }
    }
    unresolved.extend(outcome.unresolved);

    let mut findings = validate::findings(playbook);
    for key in readings.keys().filter(|k| !declared.iter().any(|d| d == k)) {
        findings.push(json!({
            "code": "sensor_not_declared",
            "name": key,
            "message": format!("reading `{key}` is not in sensor_requirements and was not used"),
        }));
    }

    Ok(json!({
        "mode": "advisory",
        "advisory_only": true,
        "playbook_id": playbook.get("id"),
        "playbook_version": playbook.get("version"),
        "sensors": {"missing_required": missing_sensors},
        "features": feature_results.iter().map(crate::features::FeatureResult::to_json).collect::<Vec<_>>(),
        "baseline": baseline,
        "failure_modes": failure_modes.iter().map(rules::FailureModeResult::to_json).collect::<Vec<_>>(),
        "recommendations": outcome.rules,
        "missing_evidence": outcome.missing,
        "unresolved": unresolved,
        "findings": findings,
    }))
}

pub(crate) fn finite_number(value: &Value, what: &str) -> Result<f64, Refusal> {
    finite(value, what)
}

pub(crate) fn is_present(value: &Value) -> bool {
    present(value)
}

pub(crate) fn bad(message: impl Into<String>) -> Refusal {
    invalid(message)
}
