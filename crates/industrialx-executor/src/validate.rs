//! Playbook validation: the repo's JSON Schema plus its cross-reference checks,
//! and the reference findings the checks deliberately do not enforce.

use std::sync::OnceLock;

use jsonschema::Validator;
use serde_json::{Value, json};

const SCHEMA_TEXT: &str = include_str!("../../../playbooks/playbook.schema.json");

fn validator() -> Result<&'static Validator, String> {
    static VALIDATOR: OnceLock<Result<Validator, String>> = OnceLock::new();
    VALIDATOR
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(SCHEMA_TEXT)
                .map_err(|error| format!("embedded playbook schema is not JSON: {error}"))?;
            jsonschema::validator_for(&schema)
                .map_err(|error| format!("embedded playbook schema does not compile: {error}"))
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn error(code: &str, field: &str, message: String) -> Value {
    json!({"code": code, "field": field, "message": message})
}

/// Schema and cross-reference errors for a playbook, empty when it is valid.
pub fn errors(playbook: &Value) -> Vec<Value> {
    let validator = match validator() {
        Ok(validator) => validator,
        Err(message) => return vec![error("executor.schema_unavailable", "", message)],
    };
    let mut found: Vec<Value> = validator
        .iter_errors(playbook)
        .map(|e| {
            error(
                "playbook.schema",
                &e.instance_path.to_string(),
                e.to_string(),
            )
        })
        .collect();
    if found.is_empty() {
        found.extend(
            industrial_asset_x::playbook_ref_diagnostics(playbook)
                .into_iter()
                .map(|d| error(d.code, &d.field, d.message)),
        );
    }
    found
}

fn strings<'a>(playbook: &'a Value, key: &str) -> Vec<&'a str> {
    playbook
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

/// Known gaps in the reference playbook. They are reported, never repaired:
/// the playbook still validates, and nothing here rewrites a name.
pub fn findings(playbook: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    let mapped = strings(
        playbook.get("baseline_scoring").unwrap_or(&Value::Null),
        "evidence_mapping",
    );
    for rule in playbook
        .get("recommendation_rules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let rule_id = rule.get("id").and_then(Value::as_str).unwrap_or("?");
        for item in strings(rule, "evidence_required") {
            if !mapped.contains(&item) {
                out.push(json!({
                    "code": "evidence_not_in_baseline_mapping",
                    "rule_id": rule_id,
                    "evidence": item,
                    "message": format!(
                        "rule `{rule_id}` requires evidence `{item}`, which baseline_scoring.evidence_mapping does not list"
                    ),
                }));
            }
        }
    }
    let template = strings(
        playbook.get("asset_template").unwrap_or(&Value::Null),
        "components",
    );
    let components = strings(playbook, "components");
    if playbook.get("components").is_some() {
        for name in &template {
            if !components.contains(name) {
                out.push(mismatch("asset_template.components", name, &components));
            }
        }
        for name in &components {
            if !template.contains(name) {
                out.push(mismatch("components", name, &template));
            }
        }
    }
    out
}

fn mismatch(origin: &str, name: &str, other: &[&str]) -> Value {
    let lookalike = other
        .iter()
        .find(|o| o.replace('-', "_") == name.replace('-', "_"));
    let hint = lookalike
        .map(|o| format!("; `{o}` differs only in `-` versus `_`"))
        .unwrap_or_default();
    json!({
        "code": "component_name_mismatch",
        "field": origin,
        "name": name,
        "message": format!("component `{name}` from {origin} has no exact match in the other list{hint}"),
    })
}
