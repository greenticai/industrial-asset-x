//! Greentic Industrial-X Designer extension.
//!
//! Authoring tools for Industrial-X playbooks: a catalogue of what ships, a
//! deterministic starter generator and the validator that is the dialect's
//! authority for the playbook body.
#![allow(clippy::used_underscore_items)]

#[rustfmt::skip]
#[allow(warnings)]
pub mod bindings;

pub mod advisory;
pub mod catalog;
pub mod generate;
pub mod refs;
pub mod tool_meta;
pub mod validate;

pub use bindings::exports;

use bindings::exports::greentic::extension_base::{lifecycle, manifest};
use bindings::exports::greentic::extension_design::tools;
use bindings::greentic::extension_base::types;
use serde::Deserialize;

pub struct Component;

impl manifest::Guest for Component {
    fn get_identity() -> types::ExtensionIdentity {
        types::ExtensionIdentity {
            id: "greentic.industrial-x-designer".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            kind: types::Kind::Design,
        }
    }

    fn get_offered() -> Vec<types::CapabilityRef> {
        vec![types::CapabilityRef {
            id: "greentic:industrial-x-designer/playbook-authoring".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }]
    }

    fn get_required() -> Vec<types::CapabilityRef> {
        vec![]
    }
}

impl lifecycle::Guest for Component {
    fn init(_config_json: String) -> Result<(), types::ExtensionError> {
        Ok(())
    }

    fn shutdown() {}
}

#[derive(Deserialize)]
struct ValidateInput {
    json: String,
}

impl tools::Guest for Component {
    fn list_tools() -> Vec<tools::ToolDefinition> {
        tool_meta::all_tools()
            .into_iter()
            .map(|meta| tools::ToolDefinition {
                name: meta.name.into(),
                description: meta.description.into(),
                input_schema_json: meta.input_schema_json.into(),
                output_schema_json: Some(meta.output_schema_json.into()),
                capabilities: Some(meta.capabilities),
                agentic_worker_metadata: Some(meta.agentic_worker_metadata.into()),
            })
            .collect()
    }

    fn invoke_tool(name: String, args_json: String) -> Result<String, types::ExtensionError> {
        let invalid = types::ExtensionError::InvalidInput;
        let output = match name.as_str() {
            tool_meta::LIST_CATALOG_TOOL => {
                catalog::list(&decode(&name, &args_json)?).map_err(invalid)?
            }
            tool_meta::GENERATE_PLAYBOOK_TOOL => {
                generate::generate(&decode(&name, &args_json)?).map_err(invalid)?
            }
            tool_meta::VALIDATE_PLAYBOOK_TOOL => {
                let input: ValidateInput = decode(&name, &args_json)?;
                serde_json::to_value(validate::validate(&input.json)).map_err(|error| {
                    types::ExtensionError::Internal(format!("encode verdict: {error}"))
                })?
            }
            other => return Err(invalid(format!("unknown tool: {other}"))),
        };
        serde_json::to_string(&output)
            .map_err(|error| types::ExtensionError::Internal(format!("encode output: {error}")))
    }
}

fn decode<T: serde::de::DeserializeOwned>(
    tool: &str,
    args_json: &str,
) -> Result<T, types::ExtensionError> {
    serde_json::from_str(args_json).map_err(|error| {
        types::ExtensionError::InvalidInput(format!("decode {tool} input: {error}"))
    })
}

#[cfg(target_family = "wasm")]
bindings::export!(Component with_types_in bindings);
