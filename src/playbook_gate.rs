//! Release gate for `playbook.json` (design §8.3).
//!
//! A playbook, including one drafted from a prompt, is data an engineer reviews.
//! The gate refuses what a schema cannot: control actions, unbounded historian
//! reads, invented ids, undeclared evidence, and unapproved providers or
//! permissions. Every rule is opt-in by field, so a playbook that declares none
//! of the 1.1 fields (`actions`, `historian_queries`, `providers`,
//! `evidence_requirements`) is judged exactly as before.
//!
//! Rules and codes: `docs/playbook-release-gate.md`.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use super::{Diagnostic, display_path};

const CATALOG_TEXT: &str = include_str!("../playbooks/provider-catalog.json");

/// Action kinds that only advise or draft. Nothing here acts on equipment.
pub const ALLOWED_ACTION_KINDS: &[&str] = &["recommendation", "draft_work_request"];

/// Kinds that name a control or alarm-handling operation. They get their own
/// code so the refusal says what was attempted, not just "unknown kind".
const CONTROL_ACTION_KINDS: &[&str] = &[
    "write_setpoint",
    "plc_write",
    "scada_write",
    "control_command",
    "acknowledge_alarm",
    "suppress_alarm",
    "execute_command",
];

/// Upper bound on a historian look-back window: 30 days.
pub const MAX_WINDOW_SECONDS: u64 = 30 * 24 * 60 * 60;
/// Upper bound on points one historian query may return.
pub const MAX_POINTS_LIMIT: u64 = 100_000;

fn array<'a>(json: &'a Value, key: &str) -> &'a [Value] {
    json.get(key)
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

fn strings<'a>(json: &'a Value, key: &str) -> BTreeSet<&'a str> {
    array(json, key).iter().filter_map(Value::as_str).collect()
}

fn ids(items: &[Value]) -> BTreeSet<&str> {
    items
        .iter()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .collect()
}

/// Parses `<integer><s|m|h|d>` into seconds. `None` for anything else, zero, or overflow.
pub fn window_seconds(window: &str) -> Option<u64> {
    let unit = window.chars().last()?;
    let digits = window.get(..window.len() - unit.len_utf8())?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let count: u64 = digits.parse().ok()?;
    let factor = match unit {
        's' => 1,
        'm' => 60,
        'h' => 3_600,
        'd' => 86_400,
        _ => return None,
    };
    count.checked_mul(factor).filter(|seconds| *seconds > 0)
}

struct Gate<'a> {
    location: String,
    out: &'a mut Vec<Diagnostic>,
}

impl Gate<'_> {
    fn reject(&mut self, code: &'static str, field: String, message: String) {
        self.out
            .push(Diagnostic::new(code, self.location.clone(), field, message));
    }
}

pub(crate) fn check(path: &Path, json: &Value, diagnostics: &mut Vec<Diagnostic>) {
    let mut gate = Gate {
        location: display_path(path),
        out: diagnostics,
    };
    let providers = declared_providers(&mut gate, json);
    historian_queries(&mut gate, json, &providers);
    actions(&mut gate, json);
    evidence(&mut gate, json);
}

fn catalog() -> Result<Vec<(String, BTreeSet<String>)>, String> {
    let parsed: Value = serde_json::from_str(CATALOG_TEXT).map_err(|e| e.to_string())?;
    Ok(array(&parsed, "providers")
        .iter()
        .filter_map(|p| {
            let id = p.get("id")?.as_str()?.to_owned();
            let permissions = array(p, "permissions")
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            Some((id, permissions))
        })
        .collect())
}

/// Checks `providers[]` against the repo catalog and returns the declared ids.
fn declared_providers<'a>(gate: &mut Gate<'_>, json: &'a Value) -> BTreeSet<&'a str> {
    let declared = array(json, "providers");
    if declared.is_empty() {
        return BTreeSet::new();
    }
    let catalog = match catalog() {
        Ok(catalog) => catalog,
        Err(error) => {
            gate.reject(
                "playbook.gate_catalog_unavailable",
                "providers".into(),
                format!("the provider catalog could not be read, so providers cannot be approved: {error}"),
            );
            return ids(declared);
        }
    };
    for (index, provider) in declared.iter().enumerate() {
        let Some(id) = provider.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some((_, allowed)) = catalog.iter().find(|(known, _)| known == id) else {
            gate.reject(
                "playbook.provider_not_approved",
                format!("providers[{index}].id"),
                format!("provider `{id}` is not in the approved provider catalog"),
            );
            continue;
        };
        for (p_index, permission) in array(provider, "permissions").iter().enumerate() {
            let Some(permission) = permission.as_str() else {
                continue;
            };
            if !allowed.contains(permission) {
                gate.reject(
                    "playbook.permission_not_allowed",
                    format!("providers[{index}].permissions[{p_index}]"),
                    format!(
                        "provider `{id}` is not approved for permission `{permission}`; playbooks may only request read-only permissions the catalog grants"
                    ),
                );
            }
        }
    }
    ids(declared)
}

fn historian_queries(gate: &mut Gate<'_>, json: &Value, providers: &BTreeSet<&str>) {
    let signals: BTreeSet<&str> = array(json, "sensor_requirements")
        .iter()
        .filter_map(|s| s.get("name").and_then(Value::as_str))
        .collect();
    for (index, query) in array(json, "historian_queries").iter().enumerate() {
        let id = query.get("id").and_then(Value::as_str).unwrap_or("?");
        let at = |name: &str| format!("historian_queries[{index}].{name}");

        match query.get("window") {
            None => gate.reject(
                "playbook.historian_query_unbounded",
                at("window"),
                format!("historian query `{id}` has no `window`; an unbounded historian query is refused"),
            ),
            Some(value) => match value.as_str().and_then(window_seconds) {
                None => gate.reject(
                    "playbook.historian_window_invalid",
                    at("window"),
                    format!("historian query `{id}` window must look like `15m`, `6h` or `7d` and be greater than zero"),
                ),
                Some(seconds) if seconds > MAX_WINDOW_SECONDS => gate.reject(
                    "playbook.historian_window_too_large",
                    at("window"),
                    format!("historian query `{id}` window exceeds the 30d limit"),
                ),
                Some(_) => {}
            },
        }
        match query.get("max_points") {
            None => gate.reject(
                "playbook.historian_query_unbounded",
                at("max_points"),
                format!("historian query `{id}` has no `max_points`; an unbounded historian query is refused"),
            ),
            Some(value) => match value.as_u64() {
                Some(n) if (1..=MAX_POINTS_LIMIT).contains(&n) => {}
                _ => gate.reject(
                    "playbook.historian_max_points_invalid",
                    at("max_points"),
                    format!("historian query `{id}` max_points must be an integer from 1 to {MAX_POINTS_LIMIT}"),
                ),
            },
        }
        if let Some(signal) = query.get("signal").and_then(Value::as_str)
            && !signals.contains(signal)
        {
            gate.reject(
                "playbook.invented_signal",
                at("signal"),
                format!("historian query `{id}` reads signal `{signal}`, which sensor_requirements does not declare"),
            );
        }
        if let Some(provider) = query.get("provider").and_then(Value::as_str)
            && !providers.contains(provider)
        {
            gate.reject(
                "playbook.invented_provider",
                at("provider"),
                format!("historian query `{id}` uses provider `{provider}`, which the playbook's providers do not declare"),
            );
        }
    }
}

fn actions(gate: &mut Gate<'_>, json: &Value) {
    let rules = ids(array(json, "recommendation_rules"));
    let failure_modes = strings(json, "failure_modes");
    let mut components = strings(json, "components");
    components.extend(strings(
        json.get("asset_template").unwrap_or(&Value::Null),
        "components",
    ));
    for (index, action) in array(json, "actions").iter().enumerate() {
        let id = action.get("id").and_then(Value::as_str).unwrap_or("?");
        let at = |name: &str| format!("actions[{index}].{name}");
        if let Some(kind) = action.get("kind").and_then(Value::as_str) {
            if CONTROL_ACTION_KINDS.contains(&kind) {
                gate.reject(
                    "playbook.action_control_forbidden",
                    at("kind"),
                    format!("action `{id}` has kind `{kind}`, which would act on equipment or alarms; playbooks are advisory and never write to PLC or SCADA"),
                );
            } else if !ALLOWED_ACTION_KINDS.contains(&kind) {
                gate.reject(
                    "playbook.action_kind_unsupported",
                    at("kind"),
                    format!(
                        "action `{id}` has unsupported kind `{kind}`; allowed kinds are {}",
                        ALLOWED_ACTION_KINDS.join(", ")
                    ),
                );
            }
        }
        if let Some(rule) = action.get("rule").and_then(Value::as_str)
            && !rules.contains(rule)
        {
            gate.reject(
                "playbook.invented_rule",
                at("rule"),
                format!("action `{id}` names recommendation rule `{rule}`, which the playbook does not declare"),
            );
        }
        if let Some(mode) = action.get("failure_mode").and_then(Value::as_str)
            && !failure_modes.contains(mode)
        {
            gate.reject(
                "playbook.invented_failure_mode",
                at("failure_mode"),
                format!("action `{id}` names failure mode `{mode}`, which failure_modes does not declare"),
            );
        }
        if let Some(component) = action.get("component").and_then(Value::as_str)
            && !components.contains(component)
        {
            gate.reject(
                "playbook.invented_component",
                at("component"),
                format!("action `{id}` names component `{component}`, which the playbook does not declare"),
            );
        }
    }
}

fn evidence(gate: &mut Gate<'_>, json: &Value) {
    if json.get("evidence_requirements").is_none() {
        return;
    }
    let declared = strings(json, "evidence_requirements");
    for (index, rule) in array(json, "recommendation_rules").iter().enumerate() {
        let id = rule.get("id").and_then(Value::as_str).unwrap_or("?");
        for item in strings(rule, "evidence_required") {
            if !declared.contains(item) {
                gate.reject(
                    "playbook.evidence_not_declared",
                    format!("recommendation_rules[{index}].evidence_required"),
                    format!("rule `{id}` requires evidence `{item}`, which evidence_requirements does not declare"),
                );
            }
        }
    }
    let mapping = json
        .get("baseline_scoring")
        .map(|b| strings(b, "evidence_mapping"))
        .unwrap_or_default();
    for item in mapping {
        if !declared.contains(item) {
            gate.reject(
                "playbook.evidence_not_declared",
                "baseline_scoring.evidence_mapping".into(),
                format!(
                    "evidence_mapping lists `{item}`, which evidence_requirements does not declare"
                ),
            );
        }
    }
}

#[cfg(test)]
#[path = "playbook_gate_tests.rs"]
mod tests;
