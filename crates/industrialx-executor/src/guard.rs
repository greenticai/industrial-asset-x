//! Refusals that happen before a request is looked at any further:
//! control requests and secret-looking values. Advisory only, no credentials.

use serde_json::Value;

const CONTROL_WORDS: &[&str] = &[
    "control",
    "command",
    "setpoint",
    "set_point",
    "actuat",
    "override",
    "write",
    "trip",
    "shutdown",
    "execute_action",
];

const SECRET_KEYS: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "apikey",
    "access_key",
    "authorization",
    "credential",
    "credentials",
    "private_key",
    "bearer",
];

const SECRET_PREFIXES: &[&str] = &["sk-", "ghp_", "gho_", "xoxb-", "xoxp-", "glpat-"];

fn mentions_control(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    CONTROL_WORDS.iter().any(|word| lower.contains(word))
}

/// A request is advisory. Anything that asks for control is refused: a
/// non-advisory `mode`, or a top-level field that names a control path.
pub fn control_request(request: &Value) -> Option<String> {
    let object = request.as_object()?;
    if let Some(mode) = object.get("mode")
        && mode.as_str() != Some("advisory")
    {
        return Some(format!("mode must be \"advisory\", got {mode}"));
    }
    object
        .keys()
        .find(|key| mentions_control(key))
        .map(|key| format!("field `{key}` asks for a control path; this executor is advisory only"))
}

fn is_secret_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SECRET_KEYS.contains(&lower.as_str())
}

fn opaque_token(text: &str) -> bool {
    text.len() >= 40
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+/=_-".contains(c))
        && text.chars().any(|c| c.is_ascii_digit())
        && text.chars().any(|c| c.is_ascii_uppercase())
        && text.chars().any(|c| c.is_ascii_lowercase())
}

fn secret_looking(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("bearer ") || lower.starts_with("basic ") {
        return true;
    }
    if text.contains("-----BEGIN") {
        return true;
    }
    let akia = text.len() == 20
        && text.starts_with("AKIA")
        && text
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    let prefixed = text.len() >= 20 && SECRET_PREFIXES.iter().any(|p| text.starts_with(p));
    let jwt = text.starts_with("eyJ") && text.len() >= 40 && text.matches('.').count() == 2;
    akia || prefixed || jwt || opaque_token(text)
}

fn scan(value: &Value, check_keys: bool, path: &str) -> Option<String> {
    match value {
        Value::String(text) if secret_looking(text) => {
            Some(format!("value at `{path}` looks like a credential"))
        }
        Value::Array(items) => items
            .iter()
            .enumerate()
            .find_map(|(i, item)| scan(item, check_keys, &format!("{path}[{i}]"))),
        Value::Object(map) => map.iter().find_map(|(key, item)| {
            if check_keys && is_secret_key(key) {
                Some(format!("field `{path}.{key}` names a credential"))
            } else {
                scan(item, check_keys, &format!("{path}.{key}"))
            }
        }),
        _ => None,
    }
}

/// Refuses literal secret-looking values anywhere in the request. Field names
/// are checked outside the playbook, whose field names the schema already fixes.
/// The refusal names the location, never the value.
pub fn secret_in_request(request: &Value) -> Option<String> {
    let Some(object) = request.as_object() else {
        return scan(request, false, "$");
    };
    object.iter().find_map(|(key, value)| {
        if is_secret_key(key) {
            return Some(format!("field `{key}` names a credential"));
        }
        scan(value, key != "playbook", key)
    })
}
