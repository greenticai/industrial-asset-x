use industrial_asset_x::{
    catalog_summary, items_to_json, list_components, list_contracts, report_to_pretty_json,
    validate_catalog, validate_store_package, value_to_pretty_json,
};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    let root =
        env::current_dir().map_err(|error| format!("could not read current dir: {error}"))?;
    match args.as_slice() {
        [] => Ok(help()),
        [one] if one == "--help" || one == "-h" => Ok(help()),
        [area, command] if area == "catalog" && command == "summary" => {
            Ok(value_to_pretty_json(&catalog_summary(&root).to_json()))
        }
        [area, command] if area == "catalog" && command == "validate" => {
            let report = validate_catalog(&root);
            let output = report_to_pretty_json(&report);
            if report.valid {
                Ok(output)
            } else {
                Err(output)
            }
        }
        [area, command, package_dir] if area == "store" && command == "validate" => {
            let report = validate_store_package(&resolve_path(&root, package_dir));
            let output = report_to_pretty_json(&report);
            if report.valid {
                Ok(output)
            } else {
                Err(output)
            }
        }
        [area, command] if area == "contracts" && command == "list" => list_contracts(&root)
            .map(|items| value_to_pretty_json(&items_to_json(&items)))
            .map_err(diagnostics_json),
        [area, command] if area == "components" && command == "list" => list_components(&root)
            .map(|items| value_to_pretty_json(&items_to_json(&items)))
            .map_err(diagnostics_json),
        _ => Err(help()),
    }
}

fn diagnostics_json(diagnostics: Vec<industrial_asset_x::Diagnostic>) -> String {
    value_to_pretty_json(&json!({
        "valid": false,
        "diagnostics": diagnostics.iter().map(|diagnostic| diagnostic.to_json()).collect::<Vec<_>>()
    }))
}

fn resolve_path(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn help() -> String {
    [
        "industrial-asset-x catalog summary",
        "industrial-asset-x catalog validate",
        "industrial-asset-x store validate <package-dir>",
        "industrial-asset-x contracts list",
        "industrial-asset-x components list",
    ]
    .join("\n")
}
