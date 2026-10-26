use industrialx_executor::{MAX_BODY_BYTES, handle};
use serde_json::{Value, json};

fn reference() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../playbooks/substation-transformer-health-basic/playbook.json"
    );
    let text = std::fs::read_to_string(path).expect("read the reference playbook");
    serde_json::from_str(&text).expect("parse the reference playbook")
}

fn post(path: &str, body: &Value) -> (u16, Value) {
    let response = handle("POST", path, body.to_string().as_bytes());
    (response.status, response.body)
}

fn readings() -> Value {
    json!({
        "oil_temperature": 60.0, "winding_temperature": 70.0, "load": 100.0,
        "current": 10.0, "voltage": 220.0, "cooling_fan_status": "on",
        "cooling_pump_status": "on", "tap_position": 3
    })
}

fn request(oil: f64, load: f64) -> Value {
    let mut readings = readings();
    readings["oil_temperature"] = json!(oil);
    readings["load"] = json!(load);
    json!({
        "playbook": reference(),
        "readings": readings,
        "features": {"temperature_rate_of_change": 0.1, "cooling_status_consistency": 1.0},
        "bindings": {"load_normalized_temperature": {"inputs": ["oil_temperature", "load"]}},
        "scoring": {"feature": "load_normalized_temperature", "band": {"lower": 0.4, "upper": 0.8}},
    })
}

fn rule<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["recommendations"]
        .as_array()
        .and_then(|rules| rules.iter().find(|r| r["rule_id"] == id))
        .unwrap_or_else(|| panic!("rule {id} missing in {body}"))
}

struct Case {
    name: &'static str,
    body: Value,
    status: u16,
    check: fn(&Value),
}

fn healthy(body: &Value) {
    assert_eq!(body["mode"], "advisory");
    assert_eq!(body["baseline"]["status"], "computed");
    assert_eq!(body["baseline"]["health_score"], 100);
    assert_eq!(body["baseline"]["risk_level"], "low");
    assert_eq!(body["features"][0]["status"], "computed");
    assert_eq!(body["sensors"]["missing_required"], json!([]));
    assert_eq!(rule(body, "inspect-cooling")["status"], "not_triggered");
}

fn degraded(body: &Value) {
    assert_eq!(body["baseline"]["risk_level"], "high");
    let cooling = rule(body, "inspect-cooling");
    assert_eq!(cooling["status"], "triggered");
    assert_eq!(cooling["trace"]["when"][1]["actual"], "high");
    assert_eq!(cooling["trace"]["evidence"][0]["present"], true);
    let missing = body["missing_evidence"].as_array().expect("missing");
    assert!(missing.iter().all(|m| m["rule_id"] != "inspect-cooling"));
}

fn missing_evidence(body: &Value) {
    assert_eq!(
        rule(body, "inspect-cooling")["status"],
        "blocked_missing_evidence"
    );
    let missing = body["missing_evidence"].as_array().expect("missing");
    assert!(
        missing
            .contains(&json!({"rule_id": "inspect-cooling", "evidence": "temperature_residual"}))
    );
    // The sensor-quality rule is unresolved (its failure mode was not asserted) but its
    // required evidence is still reported as absent.
    assert!(missing.contains(&json!({"rule_id": "review-sensor-quality", "evidence": "flatline"})));
}

fn nothing_bound(body: &Value) {
    assert_eq!(body["features"][0]["status"], "unresolved");
    assert_eq!(body["baseline"]["status"], "unresolved");
    assert_eq!(rule(body, "inspect-cooling")["status"], "unresolved");
    let unresolved = body["unresolved"].as_array().expect("unresolved list");
    assert!(unresolved.iter().any(|u| u["subject"] == "feature"));
    assert!(unresolved.iter().any(|u| u["subject"] == "failure_mode"));
}

fn zero_load(body: &Value) {
    assert_eq!(body["features"][0]["status"], "unresolved");
    assert_eq!(body["baseline"]["status"], "unresolved");
}

fn code(expected: &'static str) -> fn(&Value) {
    match expected {
        "unknown_feature" => |b| assert_eq!(b["error"]["code"], "unknown_feature"),
        "unknown_failure_mode" => |b| assert_eq!(b["error"]["code"], "unknown_failure_mode"),
        "control_refused" => |b| assert_eq!(b["error"]["code"], "control_refused"),
        "secret_refused" => |b| assert_eq!(b["error"]["code"], "secret_refused"),
        "playbook_invalid" => |b| {
            assert_eq!(b["error"]["code"], "playbook_invalid");
            assert!(!b["errors"].as_array().expect("errors").is_empty());
        },
        _ => |b| assert_eq!(b["error"]["code"], "unknown_field"),
    }
}

fn with(mut body: Value, key: &str, value: Value) -> Value {
    body[key] = value;
    body
}

fn cooling_degraded(mut body: Value, evidence: Value) -> Value {
    body["failure_modes"] = json!({"cooling system degradation": {"active": true, "score": 0.9}});
    body["evidence"] = evidence;
    body
}

#[test]
fn execute_table() {
    let healthy_request = with(
        request(60.0, 100.0),
        "failure_modes",
        json!({"cooling system degradation": {"active": false}}),
    );
    let mut broken = reference();
    broken["failure_modes"] = json!([]);
    let cases = vec![
        Case {
            name: "healthy",
            body: healthy_request,
            status: 200,
            check: healthy,
        },
        Case {
            name: "degraded",
            body: cooling_degraded(
                request(200.0, 100.0),
                json!({"cooling_status": "fan_off", "temperature_residual": 1.2}),
            ),
            status: 200,
            check: degraded,
        },
        Case {
            name: "missing evidence",
            body: cooling_degraded(request(200.0, 100.0), json!({"cooling_status": "fan_off"})),
            status: 200,
            check: missing_evidence,
        },
        Case {
            name: "nothing bound or asserted",
            body: json!({"playbook": reference(), "readings": readings()}),
            status: 200,
            check: nothing_bound,
        },
        Case {
            name: "zero load",
            body: request(60.0, 0.0),
            status: 200,
            check: zero_load,
        },
        Case {
            name: "unknown feature",
            body: with(
                request(60.0, 100.0),
                "features",
                json!({"vibration_rms": 1.0}),
            ),
            status: 422,
            check: code("unknown_feature"),
        },
        Case {
            name: "unknown binding",
            body: with(
                request(60.0, 100.0),
                "bindings",
                json!({"vibration_rms": {"inputs": []}}),
            ),
            status: 422,
            check: code("unknown_feature"),
        },
        Case {
            name: "unknown failure mode",
            body: with(
                request(60.0, 100.0),
                "failure_modes",
                json!({"gremlins": {"active": true}}),
            ),
            status: 422,
            check: code("unknown_failure_mode"),
        },
        Case {
            name: "control mode",
            body: with(request(60.0, 100.0), "mode", json!("control")),
            status: 422,
            check: code("control_refused"),
        },
        Case {
            name: "control field",
            body: with(request(60.0, 100.0), "setpoint", json!({"cooling": "on"})),
            status: 422,
            check: code("control_refused"),
        },
        Case {
            name: "secret value",
            body: with(
                request(60.0, 100.0),
                "evidence",
                json!({"note": "Bearer abcdef"}),
            ),
            status: 422,
            check: code("secret_refused"),
        },
        Case {
            name: "secret field",
            body: with(request(60.0, 100.0), "evidence", json!({"api_key": "x"})),
            status: 422,
            check: code("secret_refused"),
        },
        Case {
            name: "unknown request field",
            body: with(request(60.0, 100.0), "extra", json!(1)),
            status: 400,
            check: code("unknown_field"),
        },
        Case {
            name: "invalid playbook",
            body: with(request(60.0, 100.0), "playbook", broken),
            status: 422,
            check: code("playbook_invalid"),
        },
    ];
    for case in cases {
        let (status, body) = post("/execute", &case.body);
        assert_eq!(status, case.status, "{}: {body}", case.name);
        (case.check)(&body);
    }
}

#[test]
fn reference_findings_are_reported_not_hidden() {
    let (status, body) = post("/execute", &request(60.0, 100.0));
    assert_eq!(status, 200);
    let findings = body["findings"].as_array().expect("findings");
    let unmapped: Vec<&str> = findings
        .iter()
        .filter(|f| f["code"] == "evidence_not_in_baseline_mapping")
        .filter_map(|f| f["evidence"].as_str())
        .collect();
    for name in ["temperature_residual", "flatline", "missingness"] {
        assert!(
            unmapped.contains(&name),
            "{name} not reported in {findings:?}"
        );
    }
    assert!(!unmapped.contains(&"cooling_status"));
    let names: Vec<&str> = findings
        .iter()
        .filter(|f| f["code"] == "component_name_mismatch")
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert!(
        names.contains(&"main_tank") && names.contains(&"main-tank"),
        "{findings:?}"
    );
}

#[test]
fn validate_table() {
    let mut invalid = reference();
    invalid["id"] = json!("Not A Slug");
    let mut dangling = reference();
    dangling["features"] = json!(["load_normalized_temperature", "ghost"]);
    let cases = [
        ("reference wrapped", json!({"playbook": reference()}), true),
        ("reference bare", reference(), true),
        ("schema violation", json!({"playbook": invalid}), false),
        ("ref violation", json!({"playbook": dangling}), false),
        ("not a playbook", json!({"playbook": 5}), false),
    ];
    for (name, body, valid) in cases {
        let (status, out) = post("/validate", &body);
        assert_eq!(status, 200, "{name}: {out}");
        assert_eq!(out["valid"], valid, "{name}: {out}");
        assert_eq!(
            out["errors"].as_array().expect("errors").is_empty(),
            valid,
            "{name}"
        );
    }
    let (_, out) = post("/validate", &reference());
    assert_eq!(out["playbook_id"], "substation-transformer-health-basic");
    assert!(!out["warnings"].as_array().expect("warnings").is_empty());
}

#[test]
fn validate_answers_200_for_garbage_but_413_for_oversize() {
    let response = handle("POST", "/validate", b"{not json");
    assert_eq!(
        (response.status, &response.body["valid"]),
        (200, &json!(false))
    );
    let huge = vec![b' '; MAX_BODY_BYTES + 1];
    for path in ["/validate", "/execute"] {
        let response = handle("POST", path, &huge);
        assert_eq!(response.status, 413, "{path}");
        assert_eq!(response.body["error"]["code"], "body_too_large");
    }
}

#[test]
fn routing_is_strict() {
    assert_eq!(handle("GET", "/execute", b"").status, 405);
    assert_eq!(handle("POST", "/nope", b"{}").status, 404);
    assert_eq!(handle("POST", "/execute", b"[1]").status, 400);
    assert_eq!(
        handle("POST", "/execute", b"nope").body["error"]["code"],
        "invalid_json"
    );
}

#[test]
fn the_same_request_gives_the_same_bytes() {
    let body = request(150.0, 100.0);
    assert_eq!(post("/execute", &body), post("/execute", &body));
}
