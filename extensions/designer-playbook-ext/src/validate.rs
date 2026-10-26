//! `ix_validate_playbook`: schema, reference and advisory-only checks.

use std::sync::OnceLock;

use serde::Serialize;
use serde_json::Value;

use crate::{advisory, refs};

/// The dialect schema, embedded at build time. The repository file is the
/// single source of truth.
pub const SCHEMA_JSON: &str = include_str!("../../../playbooks/playbook.schema.json");

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Verdict {
    pub valid: bool,
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook_id: Option<String>,
    /// WASM component ids the playbook's feature implementations depend on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
}

fn schema_validator() -> Result<&'static jsonschema::Validator, String> {
    static VALIDATOR: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();
    VALIDATOR
        .get_or_init(|| {
            let schema: Value =
                serde_json::from_str(SCHEMA_JSON).map_err(|e| format!("embedded schema: {e}"))?;
            jsonschema::validator_for(&schema).map_err(|e| format!("embedded schema: {e}"))
        })
        .as_ref()
        .map_err(Clone::clone)
}

fn invalid(errors: Vec<String>) -> Verdict {
    Verdict {
        valid: false,
        errors,
        playbook_id: None,
        depends_on: None,
    }
}

/// Feature implementations that name a WASM component (a bare kebab-case id
/// ending in `-wasm`) rather than a `crate::function` path.
fn depends_on(playbook: &Value) -> Vec<String> {
    let mut ids: Vec<String> = playbook
        .get("feature_definitions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|d| d.get("implementation").and_then(Value::as_str))
        .filter(|i| !i.contains("::") && i.ends_with("-wasm"))
        .map(str::to_owned)
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn advisory_errors(playbook: &Value) -> Vec<String> {
    playbook
        .get("recommendation_rules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|rule| {
            let action = rule.get("action").and_then(Value::as_str)?;
            let id = rule.get("id").and_then(Value::as_str).unwrap_or("?");
            advisory::command_phrase(action).map(|phrase| {
                format!(
                    "advisory_only: rule `{id}` action names a command (`{phrase}`); \
                     a recommendation is advice for an engineer, never a write to a PLC, SCADA or setpoint"
                )
            })
        })
        .collect()
}

/// Validates the playbook document held in `json`.
pub fn validate(json: &str) -> Verdict {
    let playbook: Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => return invalid(vec![format!("not valid JSON: {error}")]),
    };
    let validator = match schema_validator() {
        Ok(validator) => validator,
        Err(error) => return invalid(vec![error]),
    };

    let mut errors: Vec<String> = validator
        .iter_errors(&playbook)
        .map(|e| format!("schema: {} at `{}`", e, e.instance_path))
        .collect();
    errors.extend(refs::check(&playbook));
    errors.extend(advisory_errors(&playbook));

    let valid = errors.is_empty();
    Verdict {
        valid,
        errors,
        playbook_id: playbook
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        depends_on: Some(depends_on(&playbook)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const REFERENCE: &str =
        include_str!("../../../playbooks/substation-transformer-health-basic/playbook.json");

    fn reference() -> Value {
        serde_json::from_str(REFERENCE).expect("reference parses")
    }

    #[test]
    fn the_reference_playbook_validates_clean() {
        let verdict = validate(REFERENCE);
        assert!(verdict.valid, "{:?}", verdict.errors);
        assert_eq!(
            verdict.playbook_id.as_deref(),
            Some("substation-transformer-health-basic")
        );
        assert_eq!(
            verdict.depends_on,
            Some(vec!["generic-data-quality-checker-wasm".to_owned()])
        );
    }

    #[test]
    fn malformed_json_is_reported_not_panicked() {
        let verdict = validate("{ nope");
        assert!(!verdict.valid);
        assert!(verdict.errors[0].starts_with("not valid JSON"));
        assert_eq!(verdict.playbook_id, None);
    }

    #[test]
    fn an_unknown_key_fails_the_schema() {
        let mut playbook = reference();
        playbook["steps"] = json!([]);
        let verdict = validate(&playbook.to_string());
        assert!(!verdict.valid);
        assert!(
            verdict.errors.iter().any(|e| e.starts_with("schema:")),
            "{:?}",
            verdict.errors
        );
    }

    #[test]
    fn a_rule_without_evidence_fails_the_schema() {
        let mut playbook = reference();
        playbook["recommendation_rules"][0]["evidence_required"] = json!([]);
        assert!(!validate(&playbook.to_string()).valid);
    }

    #[test]
    fn an_undeclared_failure_mode_fails_the_reference_check() {
        let mut playbook = reference();
        playbook["recommendation_rules"][0]["when"]["failure_mode"] = json!("made up");
        let verdict = validate(&playbook.to_string());
        assert!(
            verdict
                .errors
                .iter()
                .any(|e| e.contains("rule_unknown_failure_mode"))
        );
    }

    #[test]
    fn a_feature_without_a_definition_fails_the_reference_check() {
        let mut playbook = reference();
        playbook["features"]
            .as_array_mut()
            .expect("array")
            .push(json!("extra_feature"));
        let verdict = validate(&playbook.to_string());
        assert!(
            verdict
                .errors
                .iter()
                .any(|e| e.contains("feature_without_definition"))
        );
    }

    #[test]
    fn a_write_setpoint_action_is_refused() {
        let mut playbook = reference();
        playbook["recommendation_rules"][0]["action"] = json!("write setpoint 80 to PLC");
        let verdict = validate(&playbook.to_string());
        assert!(!verdict.valid);
        assert!(
            verdict
                .errors
                .iter()
                .any(|e| e.starts_with("advisory_only")),
            "{:?}",
            verdict.errors
        );
    }

    #[test]
    fn inspect_review_check_actions_are_not_refused() {
        for action in [
            "inspect cooling system",
            "review signal quality",
            "check oil level",
        ] {
            let mut playbook = reference();
            playbook["recommendation_rules"][0]["action"] = json!(action);
            let verdict = validate(&playbook.to_string());
            assert!(verdict.valid, "{action}: {:?}", verdict.errors);
        }
    }

    /// Known failures of the reference playbook that await the repository
    /// owner's decision. They must NOT be enforced here.
    #[test]
    fn the_two_undecided_checks_are_not_enforced() {
        let verdict = validate(REFERENCE);
        assert!(verdict.valid);
        let playbook = reference();
        assert_ne!(
            playbook["components"][0],
            playbook["asset_template"]["components"][0]
        );
    }
}
