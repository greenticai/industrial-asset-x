//! Baseline scoring. Only `control-band-residual` is supported, and it needs a
//! feature value plus a control band. The band is not in the playbook, so the
//! caller supplies it; without it the score is unresolved.

use std::collections::BTreeMap;

use industrialx_baseline_models::{ControlBand, assess_health};
use serde_json::{Value, json};

use crate::execute::{Refusal, bad, finite_number};

const METHOD: &str = "control-band-residual";

fn unresolved(method: Option<&str>, reason: &str) -> Value {
    json!({"status": "unresolved", "method": method, "reason": reason})
}

/// Maps a score to a level with the playbook's own thresholds: the highest
/// `min_score` the score reaches.
fn level(playbook: &Value, score: f64) -> Option<String> {
    let mut levels: Vec<(f64, &str)> = playbook
        .pointer("/baseline_scoring/risk_levels")?
        .as_array()?
        .iter()
        .filter_map(|l| Some((l.get("min_score")?.as_f64()?, l.get("level")?.as_str()?)))
        .collect();
    levels.sort_by(|a, b| b.0.total_cmp(&a.0));
    levels
        .into_iter()
        .find(|(min, _)| score >= *min)
        .map(|(_, name)| name.to_owned())
}

pub fn baseline(
    playbook: &Value,
    values: &BTreeMap<String, f64>,
    request: Option<&Value>,
) -> Result<Value, Refusal> {
    let Some(scoring) = playbook.get("baseline_scoring") else {
        return Ok(json!({"status": "not_declared"}));
    };
    let method = scoring.get("method").and_then(Value::as_str);
    if method != Some(METHOD) {
        return Ok(unresolved(
            method,
            "this executor only implements control-band-residual",
        ));
    }
    let Some(request) = request else {
        return Ok(unresolved(
            method,
            "the playbook declares no control band and the request supplied no `scoring`",
        ));
    };
    let feature = request
        .get("feature")
        .and_then(Value::as_str)
        .ok_or_else(|| bad("scoring.feature must be a feature name"))?;
    let band = request
        .get("band")
        .ok_or_else(|| bad("scoring.band is required"))?;
    let lower = finite_number(
        band.get("lower").unwrap_or(&Value::Null),
        "scoring.band.lower",
    )?;
    let upper = finite_number(
        band.get("upper").unwrap_or(&Value::Null),
        "scoring.band.upper",
    )?;
    if lower > upper {
        return Err(bad("scoring.band.lower must not exceed scoring.band.upper"));
    }
    let Some(value) = values.get(feature) else {
        return Ok(unresolved(
            method,
            &format!(
                "feature `{feature}` is unresolved or not declared, so there is nothing to score"
            ),
        ));
    };
    let assessment = assess_health(*value, ControlBand { lower, upper });
    let score = f64::from(assessment.health_score);
    let Some(risk) = level(playbook, score) else {
        return Ok(unresolved(
            method,
            "health score is below every declared risk level",
        ));
    };
    Ok(json!({
        "status": "computed",
        "method": METHOD,
        "feature": feature,
        "value": value,
        "band": {"lower": lower, "upper": upper},
        "residual": assessment.residual,
        "anomaly_score": assessment.anomaly_score,
        "health_score": assessment.health_score,
        "risk_level": risk,
    }))
}
