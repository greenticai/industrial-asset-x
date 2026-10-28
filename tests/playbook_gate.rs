//! Release-gate rules (design §8.3). Each case starts from the reference
//! playbook, which must stay valid, applies one violation, and asserts that the
//! schema still accepts the document (so the gate, not the schema, refuses it)
//! and that the gate reports exactly the expected stable code and field.

use std::fs;

use industrial_asset_x::{Diagnostic, playbook_ref_diagnostics};
use serde_json::{Value, json};

const SCHEMA: &str = "playbooks/playbook.schema.json";
const REFERENCE: &str = "playbooks/substation-transformer-health-basic/playbook.json";

fn read(path: &str) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

fn schema_errors(playbook: &Value) -> Vec<String> {
    let validator = jsonschema::validator_for(&read(SCHEMA)).expect("schema compiles");
    validator
        .iter_errors(playbook)
        .map(|e| e.to_string())
        .collect()
}

/// A reference playbook extended with every 1.1 field, all of them valid.
fn gated() -> Value {
    let mut p = read(REFERENCE);
    p["providers"] = json!([{"id": "historian-readonly", "permissions": ["read_timeseries"]}]);
    p["evidence_requirements"] = json!([
        "residual",
        "sensor_quality",
        "cooling_status",
        "temperature_residual",
        "flatline",
        "missingness"
    ]);
    p["historian_queries"] = json!([{
        "id": "oil-trend", "signal": "oil_temperature", "provider": "historian-readonly",
        "window": "7d", "max_points": 10000
    }]);
    p["actions"] = json!([
        {"id": "advise-cooling", "kind": "recommendation", "rule": "inspect-cooling",
         "component": "cooling_system", "failure_mode": "cooling system degradation"},
        {"id": "draft-wr", "kind": "draft_work_request"}
    ]);
    p
}

fn findings(playbook: &Value) -> Vec<Diagnostic> {
    playbook_ref_diagnostics(playbook)
}

/// Applies `edit`, asserts the schema still accepts it, and returns (code, field) pairs.
fn refused(edit: impl FnOnce(&mut Value)) -> Vec<(&'static str, String)> {
    let mut p = gated();
    edit(&mut p);
    assert_eq!(
        schema_errors(&p),
        Vec::<String>::new(),
        "schema must accept it"
    );
    findings(&p)
        .into_iter()
        .map(|d| (d.code, d.field))
        .collect()
}

fn expect(edit: impl FnOnce(&mut Value), code: &str, field: &str) {
    let got = refused(edit);
    assert!(
        got.iter().any(|(c, f)| *c == code && f == field),
        "expected {code} at {field}, got {got:?}"
    );
}

#[test]
fn the_reference_playbook_is_still_valid_without_any_new_field() {
    let p = read(REFERENCE);
    assert!(schema_errors(&p).is_empty());
    assert!(findings(&p).is_empty());
}

#[test]
fn a_playbook_using_every_new_field_correctly_passes() {
    let p = gated();
    assert!(schema_errors(&p).is_empty());
    assert!(findings(&p).is_empty(), "{:?}", findings(&p));
}

#[test]
fn a_playbook_without_actions_is_valid_and_advice_only() {
    let mut p = gated();
    p.as_object_mut().expect("object").remove("actions");
    assert!(findings(&p).is_empty());
}

#[test]
fn unknown_top_level_fields_are_still_refused() {
    let mut p = gated();
    p["steps"] = json!([]);
    p["permissions"] = json!(["write"]);
    assert!(!schema_errors(&p).is_empty());
}

#[test]
fn unknown_fields_inside_new_objects_are_refused() {
    for edit in [
        |p: &mut Value| p["actions"][0]["command"] = json!("trip"),
        |p: &mut Value| p["historian_queries"][0]["raw_sql"] = json!("select 1"),
        |p: &mut Value| p["providers"][0]["endpoint"] = json!("http://x"),
    ] {
        let mut p = gated();
        edit(&mut p);
        assert!(!schema_errors(&p).is_empty());
    }
}

#[test]
fn control_actions_are_refused_with_a_dedicated_code() {
    for kind in [
        "write_setpoint",
        "plc_write",
        "control_command",
        "acknowledge_alarm",
        "suppress_alarm",
    ] {
        expect(
            |p| p["actions"][1]["kind"] = json!(kind),
            "playbook.action_control_forbidden",
            "actions[1].kind",
        );
    }
}

#[test]
fn an_unknown_action_kind_is_refused() {
    expect(
        |p| p["actions"][0]["kind"] = json!("send_email"),
        "playbook.action_kind_unsupported",
        "actions[0].kind",
    );
}

#[test]
fn a_historian_query_without_a_window_is_unbounded() {
    expect(
        |p| {
            p["historian_queries"][0]
                .as_object_mut()
                .expect("object")
                .remove("window");
        },
        "playbook.historian_query_unbounded",
        "historian_queries[0].window",
    );
}

#[test]
fn a_historian_query_without_max_points_is_unbounded() {
    expect(
        |p| {
            p["historian_queries"][0]
                .as_object_mut()
                .expect("object")
                .remove("max_points");
        },
        "playbook.historian_query_unbounded",
        "historian_queries[0].max_points",
    );
}

#[test]
fn a_malformed_or_zero_window_is_refused() {
    for window in ["forever", "0h", "7 days", ""] {
        expect(
            |p| p["historian_queries"][0]["window"] = json!(window),
            "playbook.historian_window_invalid",
            "historian_queries[0].window",
        );
    }
}

#[test]
fn a_window_over_thirty_days_is_refused_and_thirty_days_passes() {
    expect(
        |p| p["historian_queries"][0]["window"] = json!("31d"),
        "playbook.historian_window_too_large",
        "historian_queries[0].window",
    );
    expect(
        |p| p["historian_queries"][0]["window"] = json!("721h"),
        "playbook.historian_window_too_large",
        "historian_queries[0].window",
    );
    let mut p = gated();
    p["historian_queries"][0]["window"] = json!("30d");
    assert!(findings(&p).is_empty());
}

#[test]
fn max_points_outside_one_to_one_hundred_thousand_is_refused() {
    for points in [0, 100_001] {
        expect(
            |p| p["historian_queries"][0]["max_points"] = json!(points),
            "playbook.historian_max_points_invalid",
            "historian_queries[0].max_points",
        );
    }
    let mut p = gated();
    p["historian_queries"][0]["max_points"] = json!(100_000);
    assert!(findings(&p).is_empty());
}

#[test]
fn a_query_for_an_undeclared_signal_is_an_invented_signal() {
    expect(
        |p| p["historian_queries"][0]["signal"] = json!("hot_spot_temperature"),
        "playbook.invented_signal",
        "historian_queries[0].signal",
    );
}

#[test]
fn a_query_naming_an_undeclared_provider_is_an_invented_provider() {
    expect(
        |p| p["historian_queries"][0]["provider"] = json!("shadow-historian"),
        "playbook.invented_provider",
        "historian_queries[0].provider",
    );
}

#[test]
fn a_provider_outside_the_catalog_is_not_approved() {
    expect(
        |p| p["providers"][0]["id"] = json!("shadow-historian"),
        "playbook.provider_not_approved",
        "providers[0].id",
    );
}

#[test]
fn a_permission_the_catalog_does_not_grant_is_refused() {
    for permission in ["write_setpoint", "read_asset_registry", "admin"] {
        expect(
            |p| p["providers"][0]["permissions"] = json!(["read_timeseries", permission]),
            "playbook.permission_not_allowed",
            "providers[0].permissions[1]",
        );
    }
}

#[test]
fn evidence_a_rule_requires_but_the_playbook_does_not_declare_is_refused() {
    expect(
        |p| {
            p["evidence_requirements"] = json!([
                "residual",
                "sensor_quality",
                "cooling_status",
                "flatline",
                "missingness"
            ]);
        },
        "playbook.evidence_not_declared",
        "recommendation_rules[0].evidence_required",
    );
}

#[test]
fn evidence_mapped_in_scoring_but_not_declared_is_refused() {
    expect(
        |p| p["baseline_scoring"]["evidence_mapping"] = json!(["residual", "vibration"]),
        "playbook.evidence_not_declared",
        "baseline_scoring.evidence_mapping",
    );
}

#[test]
fn an_action_naming_an_undeclared_rule_component_or_failure_mode_is_invented() {
    expect(
        |p| p["actions"][0]["rule"] = json!("replace-transformer"),
        "playbook.invented_rule",
        "actions[0].rule",
    );
    expect(
        |p| p["actions"][0]["component"] = json!("bushing"),
        "playbook.invented_component",
        "actions[0].component",
    );
    expect(
        |p| p["actions"][0]["failure_mode"] = json!("core meltdown"),
        "playbook.invented_failure_mode",
        "actions[0].failure_mode",
    );
}

#[test]
fn diagnostics_carry_a_readable_message_and_no_debug_noise() {
    let mut p = gated();
    p["actions"][1]["kind"] = json!("write_setpoint");
    let found = findings(&p);
    let message = &found[0].message;
    assert!(message.contains("write_setpoint") && message.contains("never write to PLC"));
    assert!(!message.contains("panicked") && !message.contains("Backtrace"));
}
