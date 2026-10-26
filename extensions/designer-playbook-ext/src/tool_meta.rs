//! Static tool metadata for the Industrial-X Designer extension.
//!
//! `describe.json` is what the Designer reads for a `greentic.ai/v2`
//! extension, so the `describe_sync` test keeps it equal to this module.

pub const LIST_CATALOG_TOOL: &str = "ix_list_authoring_catalog";
pub const GENERATE_PLAYBOOK_TOOL: &str = "ix_generate_playbook";
pub const VALIDATE_PLAYBOOK_TOOL: &str = "ix_validate_playbook";

/// Marks a tool as an authoring tool of the industrial-x dialect
/// (dialect playbook contract v1, section 4).
pub const DIALECT_CAPABILITY: &str = "cap://dialect/industrial-x/playbook-authoring";

const LIST_CATALOG_DESC: &str = "List the playbooks, sensor names, failure modes and WASM components packaged with Industrial-X.";
const GENERATE_DESC: &str = "Generate a starter Industrial-X playbook (JSON) from an objective. Placeholders only; no tags or thresholds are invented.";
const VALIDATE_DESC: &str = "Validate an Industrial-X playbook JSON: JSON Schema, reference checks and the advisory-only rule.";

const LIST_CATALOG_INPUT: &str = r#"{
  "type": "object",
  "properties": {
    "category": {
      "type": "string",
      "description": "Optional category filter: playbooks, sensors, failure_modes, components"
    }
  }
}"#;

const LIST_CATALOG_OUTPUT: &str = r#"{
  "type": "object",
  "required": ["items"],
  "properties": {
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["category", "id", "summary"],
        "properties": {
          "category": { "type": "string" },
          "id": { "type": "string" },
          "summary": { "type": "string" }
        }
      }
    }
  }
}"#;

const LIST_CATALOG_AW_META: &str = r#"{
  "usage_hint": "Use this before generating or editing an Industrial-X playbook to see which sensors, failure modes and WASM components already exist. Never reference a name that is not listed.",
  "examples": [
    { "when": "the user asks what Industrial-X can monitor", "input": {} },
    { "when": "you need the WASM components a feature may use", "input": { "category": "components" } }
  ],
  "side_effects": "read",
  "cost": "low",
  "confirmation_required": false
}"#;

const GENERATE_INPUT: &str = r#"{
  "type": "object",
  "required": ["objective"],
  "properties": {
    "objective": {
      "type": "string",
      "description": "User-facing goal for the playbook, e.g. monitor transformer cooling health"
    },
    "asset_type": {
      "type": "string",
      "description": "Optional asset type, e.g. substation transformer"
    }
  }
}"#;

const GENERATE_OUTPUT: &str = r#"{
  "type": "object",
  "required": ["playbook_id", "json", "notes"],
  "properties": {
    "playbook_id": { "type": "string" },
    "json": { "type": "string" },
    "notes": { "type": "array", "items": { "type": "string" } }
  }
}"#;

const GENERATE_AW_META: &str = r#"{
  "usage_hint": "Generate a starter draft, then replace each placeholder using ix_list_authoring_catalog. Always run ix_validate_playbook on the result. Recommendations are advice for an engineer and must never command a PLC, SCADA or setpoint.",
  "examples": [
    { "when": "the user asks for a transformer cooling playbook", "input": { "objective": "Watch transformer cooling health", "asset_type": "substation transformer" } }
  ],
  "side_effects": "write",
  "cost": "low",
  "confirmation_required": false
}"#;

const VALIDATE_INPUT: &str = r#"{
  "type": "object",
  "required": ["json"],
  "properties": {
    "json": {
      "type": "string",
      "description": "Industrial-X playbook.json document to validate"
    }
  }
}"#;

const VALIDATE_OUTPUT: &str = r#"{
  "type": "object",
  "required": ["valid", "errors"],
  "properties": {
    "valid": { "type": "boolean" },
    "errors": { "type": "array", "items": { "type": "string" } },
    "playbook_id": { "type": "string" },
    "depends_on": { "type": "array", "items": { "type": "string" } }
  }
}"#;

const VALIDATE_AW_META: &str = r#"{
  "usage_hint": "Run this on every generated or edited Industrial-X playbook before presenting it as ready. It is the authority for the playbook body: schema, references and the advisory-only rule.",
  "examples": [
    { "when": "you generated or edited playbook JSON", "input": { "json": "{\"id\":\"example\"}" } }
  ],
  "side_effects": "read",
  "cost": "low",
  "confirmation_required": false
}"#;

pub struct ToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema_json: &'static str,
    pub output_schema_json: &'static str,
    pub capabilities: Vec<String>,
    pub agentic_worker_metadata: &'static str,
}

fn tool(
    name: &'static str,
    description: &'static str,
    input_schema_json: &'static str,
    output_schema_json: &'static str,
    agentic_worker_metadata: &'static str,
) -> ToolMeta {
    ToolMeta {
        name,
        description,
        input_schema_json,
        output_schema_json,
        capabilities: vec!["agentic_worker".into(), DIALECT_CAPABILITY.into()],
        agentic_worker_metadata,
    }
}

#[must_use]
pub fn all_tools() -> Vec<ToolMeta> {
    vec![
        tool(
            LIST_CATALOG_TOOL,
            LIST_CATALOG_DESC,
            LIST_CATALOG_INPUT,
            LIST_CATALOG_OUTPUT,
            LIST_CATALOG_AW_META,
        ),
        tool(
            GENERATE_PLAYBOOK_TOOL,
            GENERATE_DESC,
            GENERATE_INPUT,
            GENERATE_OUTPUT,
            GENERATE_AW_META,
        ),
        tool(
            VALIDATE_PLAYBOOK_TOOL,
            VALIDATE_DESC,
            VALIDATE_INPUT,
            VALIDATE_OUTPUT,
            VALIDATE_AW_META,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// Wire values of the contract's `SideEffects` and `Cost` enums, mirrored
    /// because the contract crate is not a dependency. The Designer drops the
    /// whole metadata blob on an out-of-contract value, with no diagnostic.
    const SIDE_EFFECTS: [&str; 4] = ["none", "read", "write", "external"];
    const COSTS: [&str; 3] = ["low", "medium", "high"];

    #[test]
    fn every_tool_carries_both_capability_strings() {
        for tool in all_tools() {
            assert_eq!(
                tool.capabilities,
                ["agentic_worker", DIALECT_CAPABILITY],
                "{}",
                tool.name
            );
        }
    }

    #[test]
    fn metadata_uses_contract_enum_values() {
        for tool in all_tools() {
            let meta: Value =
                serde_json::from_str(tool.agentic_worker_metadata).expect("valid JSON");
            assert!(
                SIDE_EFFECTS.contains(&meta["side_effects"].as_str().expect("side_effects")),
                "{}",
                tool.name
            );
            assert!(
                COSTS.contains(&meta["cost"].as_str().expect("cost")),
                "{}",
                tool.name
            );
            assert!(meta["confirmation_required"].is_boolean(), "{}", tool.name);
        }
    }

    #[test]
    fn schemas_are_valid_json() {
        for tool in all_tools() {
            serde_json::from_str::<Value>(tool.input_schema_json).expect("input schema");
            serde_json::from_str::<Value>(tool.output_schema_json).expect("output schema");
        }
    }

    /// `describe.json` — not `all_tools()` — is what the Designer reads for a
    /// `greentic.ai/v2` extension, so both must agree or a tool silently loses
    /// its capability, description or input schema.
    #[test]
    fn describe_json_matches_all_tools() {
        let describe: Value =
            serde_json::from_str(include_str!("../describe.json")).expect("describe.json is JSON");
        let declared = describe["contributions"]["tools"]
            .as_array()
            .expect("tools array");
        let tools = all_tools();
        assert_eq!(declared.len(), tools.len());
        for tool in &tools {
            let entry = declared
                .iter()
                .find(|d| d["name"] == tool.name)
                .expect("tool declared");
            let caps: Vec<String> = entry["capabilities"]
                .as_array()
                .expect("capabilities")
                .iter()
                .map(|c| c.as_str().expect("string").to_owned())
                .collect();
            assert_eq!(
                caps, tool.capabilities,
                "{}: capabilities drifted",
                tool.name
            );
            assert_eq!(
                entry["description"].as_str(),
                Some(tool.description),
                "{}",
                tool.name
            );
            assert_eq!(
                entry["input_schema"].as_str(),
                Some(tool.input_schema_json),
                "{}",
                tool.name
            );
        }
    }
}
