use std::fs;
use std::path::Path;

use serde_json::{Value, json};

fn read(path: &str) -> Value {
    let text = fs::read_to_string(Path::new(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

fn errors(schema: &Value, instance: &Value) -> Vec<String> {
    let validator = jsonschema::validator_for(schema).expect("the playbook schema compiles");
    validator
        .iter_errors(instance)
        .map(|e| e.to_string())
        .collect()
}

const SCHEMA: &str = "playbooks/playbook.schema.json";
const REFERENCE: &str = "playbooks/substation-transformer-health-basic/playbook.json";

#[test]
fn the_reference_playbook_matches_the_schema() {
    let found = errors(&read(SCHEMA), &read(REFERENCE));
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn every_playbook_in_the_repo_matches_the_schema() {
    let schema = read(SCHEMA);
    let mut checked = 0;
    for entry in fs::read_dir("playbooks").expect("playbooks dir") {
        let path = entry.expect("entry").path().join("playbook.json");
        if path.is_file() {
            let found = errors(&schema, &read(&path.to_string_lossy()));
            assert!(found.is_empty(), "{}: {found:?}", path.display());
            checked += 1;
        }
    }
    assert!(checked >= 1, "no playbook was checked");
}

fn mutate(edit: impl FnOnce(&mut Value)) -> Vec<String> {
    let mut playbook = read(REFERENCE);
    edit(&mut playbook);
    errors(&read(SCHEMA), &playbook)
}

#[test]
fn a_step_graph_is_refused() {
    assert!(!mutate(|p| p["steps"] = json!([])).is_empty());
}

#[test]
fn a_recommendation_without_evidence_is_refused() {
    assert!(!mutate(|p| p["recommendation_rules"][0]["evidence_required"] = json!([])).is_empty());
}

#[test]
fn an_unknown_risk_level_is_refused() {
    assert!(
        !mutate(|p| p["recommendation_rules"][0]["when"]["risk_level"] = json!("critical"))
            .is_empty()
    );
}

#[test]
fn coverage_above_one_is_refused() {
    assert!(!mutate(|p| p["sensor_requirements"][0]["minimum_coverage"] = json!(1.5)).is_empty());
}

#[test]
fn a_missing_required_key_is_refused() {
    assert!(
        !mutate(|p| {
            p.as_object_mut().expect("object").remove("failure_modes");
        })
        .is_empty()
    );
}
