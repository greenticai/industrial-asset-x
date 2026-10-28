//! Guards for the prompt templates and the dialect service address asset.

use greentic_industrial_x_designer_extension::advisory::command_phrase;
use serde_json::Value;

const AUTHORING: &str = include_str!("../prompts/authoring.md");
const INVESTIGATION: &str = include_str!("../prompts/investigation.md");
const SERVICE_ASSET: &str = include_str!("../assets/dialect-service.json");
const DESCRIBE: &str = include_str!("../describe.json");
const BUILD_SH: &str = include_str!("../build.sh");

fn normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn assert_contains_all(template: &str, required: &[&str]) {
    let text = normalized(template);
    for phrase in required {
        assert!(
            text.contains(phrase),
            "template lost required text: {phrase}"
        );
    }
}

#[test]
fn authoring_template_keeps_every_prohibition() {
    assert_contains_all(
        AUTHORING,
        &[
            "using only the supplied asset and signal templates, units, limits, procedures, providers and permitted actions",
            "Do not invent tags, thresholds, failure modes, model capabilities or control commands",
            "If information is missing, return needs_clarification with exact fields",
            "Default to read-only recommendations and draft work requests",
            "Never create PLC, SCADA or setpoint writes",
        ],
    );
}

#[test]
fn authoring_template_names_every_input_placeholder() {
    for name in [
        "asset_template",
        "signal_catalogue",
        "units",
        "quality_expectations",
        "operating_modes",
        "failure_modes",
        "providers",
        "permitted_actions",
        "approval_policy",
        "target_schema",
    ] {
        assert!(
            AUTHORING.contains(&format!("{{{{{name}}}}}")),
            "missing {name}"
        );
    }
}

#[test]
fn investigation_template_keeps_every_prohibition() {
    assert_contains_all(
        INVESTIGATION,
        &[
            "using only the attached observations, model findings, operating limits, procedures and maintenance history",
            "Separate facts from inference and state uncertainty",
            "Do not claim a fault is confirmed without evidence",
            "Do not issue commands, change setpoints, suppress alarms, or bypass an approval step",
        ],
    );
}

/// The advisory-only check refuses an action that names a command. A template
/// has to name those words to forbid them, so each sentence the check flags
/// must be a prohibition (opens with a negation); any other flagged sentence
/// would be the template itself instructing a command.
#[test]
fn templates_pass_the_advisory_only_check() {
    for (name, template) in [("authoring", AUTHORING), ("investigation", INVESTIGATION)] {
        for sentence in template.split(['.', '\n']).map(str::trim) {
            if command_phrase(sentence).is_some() {
                let lower = sentence.to_lowercase();
                assert!(
                    lower.starts_with("do not ") || lower.starts_with("never "),
                    "{name}: sentence names a command without forbidding it: {sentence}"
                );
            }
        }
    }
}

fn pattern_matches(pattern: &str, url: &str) -> bool {
    let split = |s: &str| -> Option<(String, String, String)> {
        let (scheme, rest) = s.split_once("://")?;
        let (authority, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        Some((scheme.to_owned(), authority.to_owned(), path.to_owned()))
    };
    let (Some((ps, pa, pp)), Some((us, ua, up))) = (split(pattern), split(url)) else {
        return false;
    };
    let authority_ok = match pa.strip_prefix("*.") {
        Some(suffix) => ua.ends_with(&format!(".{suffix}")),
        None => pa == ua,
    };
    let path_ok = match pp.strip_suffix('*') {
        Some(prefix) => up.starts_with(prefix),
        None => pp == up,
    };
    ps == us && authority_ok && path_ok
}

#[test]
fn service_asset_has_only_a_url_that_describe_permits() {
    let asset: Value = serde_json::from_str(SERVICE_ASSET).expect("asset is JSON");
    let object = asset.as_object().expect("asset is an object");
    assert_eq!(object.keys().collect::<Vec<_>>(), ["url"]);
    let url = object["url"].as_str().expect("url is a string");

    let describe: Value = serde_json::from_str(DESCRIBE).expect("describe is JSON");
    let patterns: Vec<&str> = describe["runtime"]["permissions"]["network"]
        .as_array()
        .expect("network is a list")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        patterns.iter().any(|p| pattern_matches(p, url)),
        "{url} matches none of {patterns:?}"
    );
}

#[test]
fn only_local_addresses_are_permitted_until_a_production_host_is_set() {
    let describe: Value = serde_json::from_str(DESCRIBE).expect("describe is JSON");
    for pattern in describe["runtime"]["permissions"]["network"]
        .as_array()
        .expect("network is a list")
        .iter()
        .filter_map(Value::as_str)
    {
        assert!(pattern.starts_with("http://127.0.0.1:"), "{pattern}");
    }
}

#[test]
fn build_script_packages_the_service_asset() {
    assert!(
        BUILD_SH.contains(r#"cp assets/dialect-service.json "$STAGE/assets/dialect-service.json""#)
    );
}
