//! Failure modes and recommendation rules. The playbook maps no feature to a
//! failure mode, so a mode is only `asserted` when the caller (an upstream
//! detector) says so. A rule fires only when every `when` condition is true
//! and every required evidence item is present.

use serde_json::{Map, Value, json};

use crate::execute::{Refusal, bad, finite_number, is_present};

pub struct FailureModeResult {
    pub name: String,
    pub active: Option<bool>,
    pub score: Option<f64>,
    pub reason: Option<String>,
}

impl FailureModeResult {
    pub fn to_json(&self) -> Value {
        match self.active {
            Some(active) => json!({
                "name": self.name, "status": "asserted", "active": active, "score": self.score,
            }),
            None => json!({"name": self.name, "status": "unresolved", "reason": self.reason}),
        }
    }
}

pub fn failure_modes(
    modes: &[String],
    asserted: &Map<String, Value>,
) -> Result<Vec<FailureModeResult>, Refusal> {
    modes
        .iter()
        .map(|name| match asserted.get(name) {
            None => Ok(FailureModeResult {
                name: name.clone(),
                active: None,
                score: None,
                reason: Some(
                    "the playbook maps no feature to this failure mode and the request asserted none".into(),
                ),
            }),
            Some(entry) => {
                let active = entry
                    .get("active")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| bad(format!("failure_modes.{name}.active must be a boolean")))?;
                let score = entry
                    .get("score")
                    .map(|s| finite_number(s, &format!("failure_modes.{name}.score")))
                    .transpose()?;
                Ok(FailureModeResult { name: name.clone(), active: Some(active), score, reason: None })
            }
        })
        .collect()
}

pub struct Outcome {
    pub rules: Vec<Value>,
    pub missing: Vec<Value>,
    pub unresolved: Vec<Value>,
}

/// `Some(true/false)` when decidable, `None` when not.
struct Condition {
    field: &'static str,
    expected: String,
    actual: Option<String>,
    holds: Option<bool>,
    reason: Option<String>,
}

impl Condition {
    fn to_json(&self) -> Value {
        json!({"condition": self.field, "expected": self.expected, "actual": self.actual, "holds": self.holds})
    }
}

fn conditions(when: &Value, baseline: &Value, modes: &[FailureModeResult]) -> Vec<Condition> {
    let mut out = Vec::new();
    if let Some(expected) = when.get("failure_mode").and_then(Value::as_str) {
        let mode = modes.iter().find(|m| m.name == expected);
        let active = mode.and_then(|m| m.active);
        out.push(Condition {
            field: "failure_mode",
            expected: expected.to_owned(),
            actual: active.map(|a| if a { "active" } else { "inactive" }.to_owned()),
            holds: active,
            reason: active
                .is_none()
                .then(|| format!("failure mode `{expected}` was not asserted")),
        });
    }
    if let Some(expected) = when.get("risk_level").and_then(Value::as_str) {
        let actual = baseline.get("risk_level").and_then(Value::as_str);
        out.push(Condition {
            field: "risk_level",
            expected: expected.to_owned(),
            actual: actual.map(str::to_owned),
            holds: actual.map(|a| a == expected),
            reason: actual
                .is_none()
                .then(|| "risk level is unresolved (see baseline)".to_owned()),
        });
    }
    out
}

pub fn evaluate(
    playbook: &Value,
    baseline: &Value,
    modes: &[FailureModeResult],
    evidence: &Map<String, Value>,
) -> Result<Outcome, Refusal> {
    let mapped: Vec<&str> = playbook
        .pointer("/baseline_scoring/evidence_mapping")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut outcome = Outcome {
        rules: Vec::new(),
        missing: Vec::new(),
        unresolved: Vec::new(),
    };
    let rules = playbook
        .get("recommendation_rules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    for rule in rules {
        let id = rule.get("id").and_then(Value::as_str).unwrap_or("?");
        let conds = conditions(rule.get("when").unwrap_or(&Value::Null), baseline, modes);
        let needed: Vec<&str> = rule
            .get("evidence_required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        let trace: Vec<Value> = needed
            .iter()
            .map(|name| {
                json!({
                    "evidence": name,
                    "present": evidence.get(*name).is_some_and(is_present),
                    "in_baseline_evidence_mapping": mapped.contains(name),
                })
            })
            .collect();
        let missing: Vec<&str> = needed
            .iter()
            .copied()
            .filter(|n| !evidence.get(*n).is_some_and(is_present))
            .collect();
        let status = if conds.iter().any(|c| c.holds == Some(false)) {
            "not_triggered"
        } else if conds.iter().any(|c| c.holds.is_none()) {
            for reason in conds.iter().filter_map(|c| c.reason.as_ref()) {
                outcome
                    .unresolved
                    .push(json!({"subject": "recommendation_rule", "name": id, "reason": reason}));
            }
            for name in &missing {
                outcome
                    .missing
                    .push(json!({"rule_id": id, "evidence": name}));
            }
            "unresolved"
        } else if missing.is_empty() {
            "triggered"
        } else {
            for name in &missing {
                outcome
                    .missing
                    .push(json!({"rule_id": id, "evidence": name}));
            }
            "blocked_missing_evidence"
        };
        outcome.rules.push(json!({
            "rule_id": id,
            "action": rule.get("action"),
            "status": status,
            "trace": {"when": conds.iter().map(Condition::to_json).collect::<Vec<_>>(), "evidence": trace},
        }));
    }
    Ok(outcome)
}
