use super::*;
use serde_json::json;

#[test]
fn window_parser_accepts_bounded_durations_only() {
    assert_eq!(window_seconds("90s"), Some(90));
    assert_eq!(window_seconds("15m"), Some(900));
    assert_eq!(window_seconds("6h"), Some(21_600));
    assert_eq!(window_seconds("30d"), Some(MAX_WINDOW_SECONDS));
    for bad in ["", "d", "0m", "-5m", "5", "5w", "1.5h", " 5m", "5 m", "５m"] {
        assert_eq!(window_seconds(bad), None, "{bad:?}");
    }
    // Overflow is a refusal, not a panic.
    assert_eq!(window_seconds("99999999999999999999d"), None);
    assert_eq!(window_seconds("18446744073709551615d"), None);
}

#[test]
fn a_playbook_with_none_of_the_gate_fields_is_untouched() {
    let mut out = Vec::new();
    check(Path::new("p"), &json!({"failure_modes": ["m"]}), &mut out);
    assert!(out.is_empty());
}

#[test]
fn a_fully_declared_playbook_passes() {
    let playbook = json!({
        "sensor_requirements": [{"name": "oil_temperature"}],
        "failure_modes": ["m"],
        "components": ["tank"],
        "providers": [{"id": "historian-readonly", "permissions": ["read_timeseries"]}],
        "evidence_requirements": ["residual"],
        "baseline_scoring": {"evidence_mapping": ["residual"]},
        "recommendation_rules": [{"id": "r", "evidence_required": ["residual"], "when": {}}],
        "historian_queries": [{"id": "q", "signal": "oil_temperature", "provider": "historian-readonly", "window": "7d", "max_points": 5000}],
        "actions": [{"id": "a", "kind": "draft_work_request", "rule": "r", "component": "tank", "failure_mode": "m"}]
    });
    let mut out = Vec::new();
    check(Path::new("p"), &playbook, &mut out);
    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn the_embedded_catalog_parses_and_only_grants_read_or_draft_permissions() {
    let catalog = catalog().expect("catalog parses");
    assert!(!catalog.is_empty());
    for (id, permissions) in catalog {
        for permission in permissions {
            assert!(
                permission.starts_with("read_") || permission.starts_with("create_draft_"),
                "{id}: {permission}"
            );
        }
    }
}
