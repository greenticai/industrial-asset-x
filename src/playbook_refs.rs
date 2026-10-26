//! Cross-reference checks for `playbook.json` that a JSON Schema cannot express.
//!
//! Two checks only, both of which the reference playbook satisfies:
//! every listed feature has a definition (and the reverse), and every
//! recommendation rule names a failure mode the playbook declares.
//!
//! Deliberately NOT checked yet: that `evidence_required` items appear in
//! `baseline_scoring.evidence_mapping`, and that `asset_template.components`
//! and `components` agree. The reference playbook fails both (`temperature_residual`,
//! `flatline` and `missingness` are not mapped; `main_tank` vs `main-tank`), so
//! enforcing them needs a decision on the evidence vocabulary first.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::{Diagnostic, display_path};

fn names<'a>(json: &'a Value, key: &str, field: &str) -> BTreeSet<&'a str> {
    json.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| match field {
            "" => item.as_str(),
            name => item.get(name).and_then(Value::as_str),
        })
        .collect()
}

pub(crate) fn check(path: &Path, json: &Value, diagnostics: &mut Vec<Diagnostic>) {
    let location = display_path(path);
    let features = names(json, "features", "");
    let defined = names(json, "feature_definitions", "name");

    for missing in features.difference(&defined) {
        diagnostics.push(Diagnostic::new(
            "playbook.feature_without_definition",
            location.clone(),
            "features",
            format!("feature `{missing}` has no entry in feature_definitions"),
        ));
    }
    for unlisted in defined.difference(&features) {
        diagnostics.push(Diagnostic::new(
            "playbook.definition_not_listed",
            location.clone(),
            "feature_definitions",
            format!("feature definition `{unlisted}` is not listed in features"),
        ));
    }

    let failure_modes = names(json, "failure_modes", "");
    let rules = json
        .get("recommendation_rules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    for rule in rules {
        let rule_id = rule.get("id").and_then(Value::as_str).unwrap_or("?");
        let mode = rule
            .get("when")
            .and_then(|when| when.get("failure_mode"))
            .and_then(Value::as_str);
        if let Some(mode) = mode
            && !failure_modes.contains(mode)
        {
            diagnostics.push(Diagnostic::new(
                "playbook.rule_unknown_failure_mode",
                location.clone(),
                "recommendation_rules",
                format!("rule `{rule_id}` names failure mode `{mode}` that failure_modes does not declare"),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(value: Value) -> Vec<&'static str> {
        let mut diagnostics = Vec::new();
        check(Path::new("p/playbook.json"), &value, &mut diagnostics);
        diagnostics.into_iter().map(|d| d.code).collect()
    }

    #[test]
    fn a_consistent_playbook_has_no_findings() {
        let codes = run(json!({
            "features": ["a"],
            "feature_definitions": [{"name": "a", "implementation": "x"}],
            "failure_modes": ["m"],
            "recommendation_rules": [{"id": "r", "when": {"failure_mode": "m"}}]
        }));
        assert!(codes.is_empty(), "{codes:?}");
    }

    #[test]
    fn a_feature_without_a_definition_is_reported() {
        let codes = run(json!({"features": ["a"], "feature_definitions": []}));
        assert_eq!(codes, ["playbook.feature_without_definition"]);
    }

    #[test]
    fn a_definition_nobody_lists_is_reported() {
        let codes = run(json!({
            "features": [],
            "feature_definitions": [{"name": "a", "implementation": "x"}]
        }));
        assert_eq!(codes, ["playbook.definition_not_listed"]);
    }

    #[test]
    fn a_rule_naming_an_undeclared_failure_mode_is_reported() {
        let codes = run(json!({
            "failure_modes": ["m"],
            "recommendation_rules": [{"id": "r", "when": {"failure_mode": "other"}}]
        }));
        assert_eq!(codes, ["playbook.rule_unknown_failure_mode"]);
    }

    #[test]
    fn a_rule_with_only_a_risk_level_is_not_a_finding() {
        let codes = run(json!({
            "failure_modes": ["m"],
            "recommendation_rules": [{"id": "r", "when": {"risk_level": "high"}}]
        }));
        assert!(codes.is_empty(), "{codes:?}");
    }
}
