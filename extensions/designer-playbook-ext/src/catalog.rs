//! `ix_list_authoring_catalog`: what the repository packages, read from the
//! data embedded at build time.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Value, json};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_data.rs"));
}

#[derive(Debug, Default, Deserialize)]
pub struct ListCatalogInput {
    pub category: Option<String>,
}

pub const CATEGORIES: &[&str] = &["playbooks", "sensors", "failure_modes", "components"];

struct Item {
    category: &'static str,
    id: String,
    summary: String,
}

fn parse(sources: &[&str]) -> Vec<Value> {
    sources
        .iter()
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect()
}

fn items() -> Vec<Item> {
    let mut items = Vec::new();
    // sensor / failure-mode name -> playbooks that use it
    let mut sensors: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    let mut modes: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for playbook in parse(embedded::PLAYBOOKS) {
        let Some(id) = playbook.get("id").and_then(Value::as_str) else {
            continue;
        };
        let asset = playbook
            .pointer("/asset_template/type")
            .and_then(Value::as_str)
            .unwrap_or("?");
        items.push(Item {
            category: "playbooks",
            id: id.to_owned(),
            summary: format!("Packaged playbook for asset type `{asset}`."),
        });
        for sensor in playbook
            .get("sensor_requirements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(name) = sensor.get("name").and_then(Value::as_str) {
                let unit = sensor.get("unit").and_then(Value::as_str).unwrap_or("?");
                sensors
                    .entry(name.to_owned())
                    .or_insert_with(|| (unit.to_owned(), Vec::new()))
                    .1
                    .push(id.to_owned());
            }
        }
        for mode in playbook
            .get("failure_modes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(mode) = mode.as_str() {
                modes
                    .entry(mode.to_owned())
                    .or_default()
                    .push(id.to_owned());
            }
        }
    }
    for (name, (unit, used_by)) in sensors {
        items.push(Item {
            category: "sensors",
            id: name,
            summary: format!(
                "Signal of unit kind `{unit}`, used by: {}.",
                used_by.join(", ")
            ),
        });
    }
    for (mode, used_by) in modes {
        items.push(Item {
            category: "failure_modes",
            id: mode,
            summary: format!("Failure mode declared by: {}.", used_by.join(", ")),
        });
    }
    for manifest in parse(embedded::COMPONENT_MANIFESTS) {
        let Some(name) = manifest.get("name").and_then(Value::as_str) else {
            continue;
        };
        let version = manifest
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("?");
        let manifest_id = manifest.get("id").and_then(Value::as_str).unwrap_or(name);
        items.push(Item {
            category: "components",
            id: name.to_owned(),
            summary: format!("WASM component v{version} (manifest id `{manifest_id}`)."),
        });
    }
    items
}

/// Lists catalog items, optionally filtered to one category.
pub fn list(input: &ListCatalogInput) -> Result<Value, String> {
    if let Some(category) = input.category.as_deref()
        && !CATEGORIES.contains(&category)
    {
        return Err(format!(
            "unknown category `{category}`; expected one of {CATEGORIES:?}"
        ));
    }
    let items: Vec<Value> = items()
        .into_iter()
        .filter(|item| input.category.as_deref().is_none_or(|c| c == item.category))
        .map(|item| json!({"category": item.category, "id": item.id, "summary": item.summary}))
        .collect();
    Ok(json!({ "items": items }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(category: &str) -> Vec<String> {
        let out = list(&ListCatalogInput {
            category: Some(category.into()),
        })
        .expect("known category");
        out["items"]
            .as_array()
            .expect("items")
            .iter()
            .map(|i| i["id"].as_str().expect("id").to_owned())
            .collect()
    }

    #[test]
    fn lists_the_packaged_playbook() {
        assert!(ids("playbooks").contains(&"substation-transformer-health-basic".to_owned()));
    }

    #[test]
    fn lists_sensors_and_failure_modes_of_the_playbook() {
        assert!(ids("sensors").contains(&"oil_temperature".to_owned()));
        assert!(ids("failure_modes").contains(&"cooling system degradation".to_owned()));
    }

    #[test]
    fn lists_wasm_components_by_name() {
        assert!(ids("components").contains(&"generic-data-quality-checker-wasm".to_owned()));
    }

    #[test]
    fn no_filter_returns_every_category() {
        let out = list(&ListCatalogInput::default()).expect("ok");
        let seen: std::collections::BTreeSet<&str> = out["items"]
            .as_array()
            .expect("items")
            .iter()
            .filter_map(|i| i["category"].as_str())
            .collect();
        for category in CATEGORIES {
            assert!(seen.contains(category), "{category} missing");
        }
    }

    #[test]
    fn an_unknown_category_is_an_error() {
        assert!(
            list(&ListCatalogInput {
                category: Some("nope".into())
            })
            .is_err()
        );
    }
}
