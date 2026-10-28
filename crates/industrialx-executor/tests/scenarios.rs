//! Synthetic transformer-cooling scenarios (design section 9). Every file under
//! `playbooks/substation-transformer-health-basic/scenarios/` is deterministic,
//! generic data. Step scenarios run through `/execute`; gate cases apply one
//! violation to the reference playbook and must be refused by the release gate
//! with a stable code and field through `/validate`.

use std::fs;
use std::path::PathBuf;

use industrialx_executor::handle;
use serde_json::{Value, json};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../playbooks/substation-transformer-health-basic")
}

fn read(path: PathBuf) -> Value {
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn post(path: &str, body: &Value) -> (u16, Value) {
    let r = handle("POST", path, body.to_string().as_bytes());
    (r.status, r.body)
}

fn rule<'a>(out: &'a Value, id: &str) -> &'a Value {
    out["recommendations"]
        .as_array()
        .and_then(|r| r.iter().find(|x| x["rule_id"] == id))
        .unwrap_or_else(|| panic!("rule {id} missing in {out}"))
}

/// Evidence the rule actually found present in its trace.
fn cited(out: &Value, rule_id: &str) -> Vec<String> {
    let mut names: Vec<String> = rule(out, rule_id)["trace"]["evidence"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|e| e["present"] == true)
        .filter_map(|e| e["evidence"].as_str().map(str::to_owned))
        .collect();
    names.sort();
    names
}

fn check_step(name: &str, step: &Value, out: &Value, status: u16) {
    let ctx = format!("{name}/{}: {out}", step["label"]);
    let expect = &step["expect"];
    assert_eq!(
        status,
        expect["status"].as_u64().unwrap_or(200) as u16,
        "{ctx}"
    );
    if let Some(risk) = expect.get("baseline_risk") {
        assert_eq!(&out["baseline"]["risk_level"], risk, "{ctx}");
    }
    if let Some(s) = expect.get("baseline_status") {
        assert_eq!(&out["baseline"]["status"], s, "{ctx}");
    }
    for (id, want) in expect["rules"].as_object().into_iter().flatten() {
        assert_eq!(&rule(out, id)["status"], want, "{ctx} rule {id}");
    }
    if let Some(want) = expect.get("cited_evidence") {
        let rule_id = expect["rules"]
            .as_object()
            .and_then(|r| r.keys().next())
            .unwrap_or_else(|| panic!("{ctx}: cited_evidence needs a rule"));
        let mut want: Vec<String> = want
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        want.sort();
        assert_eq!(cited(out, rule_id), want, "{ctx}");
    }
    if let Some(m) = expect.get("missing_required_sensors") {
        assert_eq!(&out["sensors"]["missing_required"], m, "{ctx}");
    }
    if let Some(m) = expect.get("missing_evidence") {
        for item in m.as_array().into_iter().flatten() {
            assert!(
                out["missing_evidence"]
                    .as_array()
                    .is_some_and(|a| a.contains(item)),
                "{ctx} lacks {item}"
            );
        }
    }
}

fn scenario_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir().join("scenarios"))
        .expect("scenarios dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_step_scenario_runs_to_its_expected_branch() {
    let playbook = read(dir().join("playbook.json"));
    let mut steps = 0;
    for file in scenario_files() {
        let doc = read(file);
        for step in doc["steps"].as_array().into_iter().flatten() {
            let mut body = step["request"].clone();
            body["playbook"] = playbook.clone();
            let (status, out) = post("/execute", &body);
            check_step(doc["scenario"].as_str().unwrap_or("?"), step, &out, status);
            steps += 1;
        }
    }
    assert!(steps >= 8, "only {steps} steps ran");
}

#[test]
fn the_six_required_scenarios_exist() {
    let names: Vec<String> = scenario_files()
        .iter()
        .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_owned))
        .collect();
    for want in [
        "healthy",
        "degraded",
        "sensor_failure",
        "missing_data",
        "boundary",
        "recovery",
        "gate_rejections",
    ] {
        assert!(
            names.iter().any(|n| n == want),
            "{want} missing from {names:?}"
        );
    }
}

#[test]
fn scenario_data_is_deterministic() {
    let playbook = read(dir().join("playbook.json"));
    for file in scenario_files() {
        for step in read(file)["steps"].as_array().into_iter().flatten() {
            let mut body = step["request"].clone();
            body["playbook"] = playbook.clone();
            assert_eq!(post("/execute", &body), post("/execute", &body));
        }
    }
}

#[test]
fn gate_rejections_are_refused_with_stable_codes() {
    let reference = read(dir().join("playbook.json"));
    let cases = read(dir().join("scenarios/gate_rejections.json"));
    let (_, ok) = post("/validate", &reference);
    assert_eq!(ok["valid"], true, "{ok}");
    for case in cases["cases"].as_array().into_iter().flatten() {
        let mut playbook = reference.clone();
        for (k, v) in case["patch"].as_object().into_iter().flatten() {
            playbook[k] = v.clone();
        }
        let (status, out) = post("/validate", &json!({"playbook": playbook}));
        assert_eq!(status, 200, "{}", case["name"]);
        assert_eq!(out["valid"], false, "{}: {out}", case["name"]);
        let hit = out["errors"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|e| e["code"] == case["expect_code"])
            .unwrap_or_else(|| panic!("{} not reported: {out}", case["expect_code"]));
        assert_eq!(hit["field"], case["expect_field"], "{}", case["name"]);
    }
}
