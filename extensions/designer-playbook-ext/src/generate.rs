//! `ix_generate_playbook`: a deterministic starter draft. No model is called.
//!
//! The draft never invents a tag, threshold, failure mode or component. Where
//! the schema demands a value it gets an obvious `placeholder` and the draft is
//! marked `synthetic_demo_safe: true`, so it cannot be mistaken for a real
//! playbook. An engineer replaces the placeholders from the catalogue.

use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
pub struct GeneratePlaybookInput {
    pub objective: String,
    pub asset_type: Option<String>,
}

/// Lowercase ASCII alphanumerics joined by `sep`, at most `max` characters.
fn slug(text: &str, sep: char, max: usize) -> String {
    let mut out = String::new();
    for word in text
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        if !out.is_empty() {
            out.push(sep);
        }
        out.push_str(word);
    }
    out.truncate(max);
    out.trim_end_matches(sep).to_owned()
}

pub fn generate(input: &GeneratePlaybookInput) -> Result<Value, String> {
    if input.objective.trim().is_empty() {
        return Err("objective must not be empty".into());
    }
    let id = match slug(&input.objective, '-', 60) {
        s if s.is_empty() => "draft-playbook".to_owned(),
        s => format!("draft-{s}"),
    };
    let asset_type = input
        .asset_type
        .as_deref()
        .map(|t| slug(t, '_', 40))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| "placeholder_asset".to_owned());
    let mode = "placeholder failure mode";

    let playbook = json!({
        "id": id,
        "version": "0.0.1-draft",
        "synthetic_demo_safe": true,
        "asset_template": { "type": asset_type, "components": ["placeholder_component"] },
        "sensor_requirements": [
            { "name": "placeholder_signal", "required": true, "unit": "numeric" }
        ],
        "failure_modes": [mode],
        "recommendation_rules": [{
            "id": "review-draft",
            "action": "review this draft with a maintenance engineer",
            "evidence_required": ["placeholder_evidence"],
            "when": { "failure_mode": mode }
        }]
    });
    let json = serde_json::to_string_pretty(&playbook).map_err(|e| format!("encode draft: {e}"))?;
    Ok(json!({
        "playbook_id": id,
        "json": json,
        "notes": [
            format!("Draft for objective: {}", input.objective.trim()),
            "Every `placeholder_*` value and the failure mode are stand-ins; replace them from ix_list_authoring_catalog.",
            "No tag, threshold or failure mode was invented. Recommendations are advisory only.",
            "Run ix_validate_playbook before presenting this as ready."
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::validate;

    fn run(objective: &str, asset_type: Option<&str>) -> Value {
        generate(&GeneratePlaybookInput {
            objective: objective.into(),
            asset_type: asset_type.map(Into::into),
        })
        .expect("generates")
    }

    #[test]
    fn the_draft_passes_the_validator() {
        for (objective, asset) in [
            (
                "Watch transformer cooling health",
                Some("Substation Transformer"),
            ),
            ("!!!", None),
            ("Überwachung von Pumpen", Some("pump")),
            (&"very long objective ".repeat(20), None),
        ] {
            let out = run(objective, asset);
            let verdict = validate(out["json"].as_str().expect("json"));
            assert!(verdict.valid, "{objective}: {:?}", verdict.errors);
            assert_eq!(verdict.playbook_id.as_deref(), out["playbook_id"].as_str());
        }
    }

    #[test]
    fn the_draft_is_marked_and_carries_only_placeholders() {
        let out = run("Watch cooling", None);
        let draft: Value =
            serde_json::from_str(out["json"].as_str().expect("json")).expect("parses");
        assert_eq!(draft["synthetic_demo_safe"], json!(true));
        assert_eq!(
            draft["sensor_requirements"][0]["name"],
            "placeholder_signal"
        );
        assert!(draft.get("baseline_scoring").is_none() && draft.get("features").is_none());
    }

    #[test]
    fn generation_is_deterministic() {
        assert_eq!(
            run("Watch cooling", Some("pump")),
            run("Watch cooling", Some("pump"))
        );
    }

    #[test]
    fn an_empty_objective_is_an_error() {
        assert!(
            generate(&GeneratePlaybookInput {
                objective: "  ".into(),
                asset_type: None
            })
            .is_err()
        );
    }
}
