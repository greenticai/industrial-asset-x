use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn component_fixtures_match_rust_logic() {
    assert_signal_profiler_fixture();
    assert_data_quality_fixture();
    assert_feature_extractor_fixture();
    assert_health_scorer_fixture();
    assert_synthetic_generator_fixture();
}

fn read_fixture(component: &str, name: &str) -> Value {
    let path = Path::new("components")
        .join(component)
        .join("fixtures")
        .join(name);
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn number_array(value: &Value, field: &str) -> Vec<f64> {
    value[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap())
        .collect()
}

fn assert_signal_profiler_fixture() {
    let input = read_fixture("generic-signal-profiler-wasm", "input.json");
    let expected = read_fixture("generic-signal-profiler-wasm", "expected-output.json");
    let values = number_array(&input, "values");
    let (min, max, flatlines) = generic_signal_profiler_wasm::profile(&values);
    assert_eq!(min, expected["min"].as_f64().unwrap());
    assert_eq!(max, expected["max"].as_f64().unwrap());
    assert_eq!(
        flatlines as u64,
        expected["flatline_periods"].as_u64().unwrap()
    );
}

fn assert_data_quality_fixture() {
    let input = read_fixture("generic-data-quality-checker-wasm", "input.json");
    let expected = read_fixture("generic-data-quality-checker-wasm", "expected-output.json");
    let actual = generic_data_quality_checker_wasm::usable_for_baseline(
        input["missing_ratio"].as_f64().unwrap(),
        input["coverage_ratio"].as_f64().unwrap(),
    );
    assert_eq!(actual, expected["usable_for_baseline"].as_bool().unwrap());
}

fn assert_feature_extractor_fixture() {
    let input = read_fixture("generic-feature-extractor-wasm", "input.json");
    let expected = read_fixture("generic-feature-extractor-wasm", "expected-output.json");
    let values = number_array(&input, "values");
    let actual = generic_feature_extractor_wasm::extract(&values);
    let expected = number_array(&expected, "features");
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() < 0.000001);
    }
}

fn assert_health_scorer_fixture() {
    let input = read_fixture("generic-health-scorer-wasm", "input.json");
    let expected = read_fixture("generic-health-scorer-wasm", "expected-output.json");
    let (health_score, _) = generic_health_scorer_wasm::score(
        input["value"].as_f64().unwrap(),
        input["lower"].as_f64().unwrap(),
        input["upper"].as_f64().unwrap(),
    );
    assert_eq!(
        health_score as u64,
        expected["health_score"].as_u64().unwrap()
    );
}

fn assert_synthetic_generator_fixture() {
    let input = read_fixture("rule-based-synthetic-generator-wasm", "input.json");
    let expected = read_fixture(
        "rule-based-synthetic-generator-wasm",
        "expected-output.json",
    );
    let actual = rule_based_synthetic_generator_wasm::generate_flatline(
        input["length"].as_u64().unwrap() as usize,
    );
    assert_eq!(actual, number_array(&expected, "values"));
}
