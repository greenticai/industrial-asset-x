//! Cross-reference checks a JSON Schema cannot express.
//!
//! This is a copy of the two checks in the repository's `src/playbook_refs.rs`
//! (feature/definition agreement, and rule failure modes being declared). The
//! extension cannot depend on the root crate, whose module is private. The test
//! `diagnostic_codes_stay_in_step_with_the_root_crate` keeps the two apart only
//! as far as their diagnostic codes; if the root checks change, change these.
//!
//! Deliberately NOT checked, because the repository owner has not decided yet:
//! that `evidence_required` items appear in `baseline_scoring.evidence_mapping`,
//! and that `asset_template.components` agrees with `components` (`main_tank`
//! vs `main-tank`). The reference playbook fails both. Do not enforce them here
//! before the root checks do.

use std::collections::BTreeSet;

use serde_json::Value;

pub const CODES: &[&str] = &[
    "playbook.feature_without_definition",
    "playbook.definition_not_listed",
    "playbook.rule_unknown_failure_mode",
];

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

/// Returns one human-readable error per violated reference.
pub fn check(json: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let features = names(json, "features", "");
    let defined = names(json, "feature_definitions", "name");

    for missing in features.difference(&defined) {
        errors.push(format!(
            "{}: feature `{missing}` has no entry in feature_definitions",
            CODES[0]
        ));
    }
    for unlisted in defined.difference(&features) {
        errors.push(format!(
            "{}: feature definition `{unlisted}` is not listed in features",
            CODES[1]
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
            errors.push(format!(
                "{}: rule `{rule_id}` names failure mode `{mode}` that failure_modes does not declare",
                CODES[2]
            ));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_consistent_playbook_has_no_findings() {
        let errors = check(&json!({
            "features": ["a"],
            "feature_definitions": [{"name": "a", "implementation": "x"}],
            "failure_modes": ["m"],
            "recommendation_rules": [{"id": "r", "when": {"failure_mode": "m"}}]
        }));
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn a_feature_without_a_definition_is_reported() {
        let errors = check(&json!({"features": ["a"], "feature_definitions": []}));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with(CODES[0]));
    }

    #[test]
    fn a_definition_nobody_lists_is_reported() {
        let errors = check(&json!({
            "features": [],
            "feature_definitions": [{"name": "a", "implementation": "x"}]
        }));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with(CODES[1]));
    }

    #[test]
    fn a_rule_naming_an_undeclared_failure_mode_is_reported() {
        let errors = check(&json!({
            "failure_modes": ["m"],
            "recommendation_rules": [{"id": "r", "when": {"failure_mode": "other"}}]
        }));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].starts_with(CODES[2]));
    }

    #[test]
    fn a_rule_with_only_a_risk_level_is_not_a_finding() {
        let errors = check(&json!({
            "failure_modes": ["m"],
            "recommendation_rules": [{"id": "r", "when": {"risk_level": "high"}}]
        }));
        assert!(errors.is_empty(), "{errors:?}");
    }

    /// The extension copies the root crate's checks; this fails when the root
    /// source stops emitting one of the codes the copy reports.
    #[test]
    fn diagnostic_codes_stay_in_step_with_the_root_crate() {
        let root = include_str!("../../../src/playbook_refs.rs");
        for code in CODES {
            assert!(
                root.contains(code),
                "root playbook_refs.rs no longer has {code}"
            );
        }
        // Count of distinct `playbook.` codes in the root, so a NEW root check
        // also fails here until it is mirrored.
        let in_root = root
            .split('"')
            .filter(|s| s.starts_with("playbook.") && !s.contains(' ') && !s.contains('/'))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            in_root.len(),
            CODES.len(),
            "root has codes the extension lacks: {in_root:?}"
        );
    }
}
