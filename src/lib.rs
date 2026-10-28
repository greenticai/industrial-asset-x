use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

mod playbook_gate;
mod playbook_refs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub path: String,
    pub field: String,
    pub message: String,
}

impl Diagnostic {
    pub fn new(
        code: &'static str,
        path: impl Into<String>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "code": self.code,
            "path": self.path,
            "field": self.field,
            "message": self.message,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn ok() -> Self {
        Self {
            valid: true,
            diagnostics: Vec::new(),
        }
    }

    pub fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            valid: diagnostics.is_empty(),
            diagnostics,
        }
    }

    pub fn merge(&mut self, mut other: ValidationReport) {
        self.diagnostics.append(&mut other.diagnostics);
        self.valid = self.diagnostics.is_empty();
    }

    pub fn to_json(&self) -> Value {
        json!({
            "valid": self.valid,
            "diagnostics": self.diagnostics.iter().map(Diagnostic::to_json).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogSummary {
    pub contracts: usize,
    pub components: usize,
    pub ops: usize,
    pub flows: usize,
    pub views: usize,
    pub playbooks: usize,
    pub store_examples: usize,
}

impl CatalogSummary {
    pub fn to_json(&self) -> Value {
        json!({
            "contracts": self.contracts,
            "components": self.components,
            "ops": self.ops,
            "flows": self.flows,
            "views": self.views,
            "playbooks": self.playbooks,
            "store_examples": self.store_examples,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogItem {
    pub id: String,
    pub path: String,
}

impl CatalogItem {
    pub fn to_json(&self) -> Value {
        json!({ "id": self.id, "path": self.path })
    }
}

pub fn catalog_summary(root: &Path) -> CatalogSummary {
    CatalogSummary {
        contracts: find_named_files(&root.join("contracts"), "contract.json").len(),
        components: find_named_files(&root.join("components"), "component.manifest.json").len(),
        ops: find_named_files(&root.join("ops"), "manifest.json").len(),
        flows: find_named_files(&root.join("flows"), "flow.json").len(),
        views: find_named_files(&root.join("views"), "view.json").len(),
        playbooks: find_named_files(&root.join("playbooks"), "playbook.json").len(),
        store_examples: immediate_dirs(&root.join("store/examples")).len(),
    }
}

pub fn list_contracts(root: &Path) -> Result<Vec<CatalogItem>, Vec<Diagnostic>> {
    list_json_items(
        &root.join("contracts"),
        "contract.json",
        &["manifest", "id"],
    )
}

pub fn list_components(root: &Path) -> Result<Vec<CatalogItem>, Vec<Diagnostic>> {
    list_json_items(&root.join("components"), "component.manifest.json", &["id"])
}

/// Runs the cross-reference checks that the playbook JSON Schema cannot express.
pub fn playbook_ref_diagnostics(json: &Value) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    playbook_refs::check(Path::new("playbook"), json, &mut diagnostics);
    playbook_gate::check(Path::new("playbook"), json, &mut diagnostics);
    diagnostics
}

pub fn validate_catalog(root: &Path) -> ValidationReport {
    let mut report = ValidationReport::ok();
    report.merge(validate_catalog_descriptor(root));
    report.merge(validate_contracts(root));
    report.merge(validate_components(root));
    report.merge(validate_ops(root));
    report.merge(validate_flows(root));
    report.merge(validate_views(root));
    report.merge(validate_playbooks(root));

    for package_dir in immediate_dirs(&root.join("store/examples")) {
        report.merge(validate_store_package(&package_dir));
    }

    report
}

pub fn validate_store_package(package_dir: &Path) -> ValidationReport {
    let mut diagnostics = Vec::new();
    let required_files = [
        "package.manifest.json",
        "license.json",
        "publisher.json",
        "validation-report.json",
        "compatibility-report.json",
        "fixtures.json",
    ];

    for file in required_files {
        let path = package_dir.join(file);
        if !path.is_file() {
            diagnostics.push(Diagnostic::new(
                "store.missing_file",
                display_path(&path),
                file,
                "required store package file is missing",
            ));
        }
    }

    let manifest_path = package_dir.join("package.manifest.json");
    let manifest = read_json(&manifest_path, &mut diagnostics);
    if let Some(manifest) = manifest.as_ref() {
        for field in [
            "id",
            "kind",
            "version",
            "package_type",
            "license",
            "publisher",
            "supported_asset_types",
            "required_sensor_types",
            "components",
            "flows",
            "views",
            "validation_report",
            "compatibility_report",
        ] {
            require_field(manifest, &manifest_path, field, &mut diagnostics);
        }
        require_string_value(
            manifest,
            &manifest_path,
            "kind",
            "storex.package",
            &mut diagnostics,
        );
    }

    for entry in find_json_files(package_dir) {
        if let Some(json) = read_json(&entry, &mut diagnostics) {
            scan_banned_terms(&entry, &json, &mut diagnostics);
        }
    }

    ValidationReport::from_diagnostics(diagnostics)
}

pub fn report_to_pretty_json(report: &ValidationReport) -> String {
    serde_json::to_string_pretty(&report.to_json()).expect("validation report should serialize")
}

pub fn value_to_pretty_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).expect("json value should serialize")
}

pub fn items_to_json(items: &[CatalogItem]) -> Value {
    Value::Array(items.iter().map(CatalogItem::to_json).collect())
}

pub fn stable_object_counts(root: &Path) -> BTreeMap<&'static str, usize> {
    let summary = catalog_summary(root);
    BTreeMap::from([
        ("components", summary.components),
        ("contracts", summary.contracts),
        ("flows", summary.flows),
        ("ops", summary.ops),
        ("playbooks", summary.playbooks),
        ("store_examples", summary.store_examples),
        ("views", summary.views),
    ])
}

pub fn validate_unique_ids(items: &[CatalogItem]) -> ValidationReport {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for item in items {
        if !seen.insert(item.id.clone()) {
            diagnostics.push(Diagnostic::new(
                "catalog.duplicate_id",
                item.path.clone(),
                "id",
                format!("duplicate id {}", item.id),
            ));
        }
    }
    ValidationReport::from_diagnostics(diagnostics)
}

fn validate_catalog_descriptor(root: &Path) -> ValidationReport {
    let mut diagnostics = Vec::new();
    let path = root.join("catalog.json");
    if let Some(json) = read_json(&path, &mut diagnostics) {
        require_field(&json, &path, "id", &mut diagnostics);
        require_field(&json, &path, "namespaces", &mut diagnostics);
        require_bool_value(
            &json,
            &path,
            "customer_specific_content_allowed",
            false,
            &mut diagnostics,
        );
    }
    ValidationReport::from_diagnostics(diagnostics)
}

fn validate_contracts(root: &Path) -> ValidationReport {
    let mut diagnostics = Vec::new();
    for path in find_named_files(&root.join("contracts"), "contract.json") {
        if let Some(json) = read_json(&path, &mut diagnostics) {
            require_path(&json, &path, &["manifest", "id"], &mut diagnostics);
            require_path(&json, &path, &["manifest", "namespace"], &mut diagnostics);
            require_path(&json, &path, &["schema", "required"], &mut diagnostics);
            require_bool_value(
                pointer_or_self(&json, &["schema"]),
                &path,
                "additionalProperties",
                false,
                &mut diagnostics,
            );
            scan_banned_terms(&path, &json, &mut diagnostics);
        }
    }
    ValidationReport::from_diagnostics(diagnostics)
}

fn validate_components(root: &Path) -> ValidationReport {
    let mut diagnostics = Vec::new();
    for path in find_named_files(&root.join("components"), "component.manifest.json") {
        if let Some(json) = read_json(&path, &mut diagnostics) {
            let canonical_greentic = json.get("operations").and_then(Value::as_array).is_some();
            if canonical_greentic {
                for field in ["id", "version", "world", "operations", "capabilities"] {
                    require_field(&json, &path, field, &mut diagnostics);
                }
                if let Some(operations) = json.get("operations").and_then(Value::as_array) {
                    for operation in operations {
                        for field in ["name", "input_schema", "output_schema"] {
                            require_field(operation, &path, field, &mut diagnostics);
                        }
                    }
                }
            } else {
                for field in ["id", "world", "input_contract", "output_contract"] {
                    require_field(&json, &path, field, &mut diagnostics);
                }
                require_bool_value(&json, &path, "deterministic", true, &mut diagnostics);
            }
            let component_dir = path.parent().unwrap_or_else(|| Path::new("."));
            for fixture in ["fixtures/input.json", "fixtures/expected-output.json"] {
                if !component_dir.join(fixture).is_file() {
                    diagnostics.push(Diagnostic::new(
                        "component.missing_fixture",
                        display_path(component_dir.join(fixture)),
                        fixture,
                        "component fixture is missing",
                    ));
                }
            }
            scan_banned_terms(&path, &json, &mut diagnostics);
        }
    }
    ValidationReport::from_diagnostics(diagnostics)
}

fn validate_ops(root: &Path) -> ValidationReport {
    let mut diagnostics = Vec::new();
    for path in find_named_files(&root.join("ops"), "manifest.json") {
        if let Some(json) = read_json(&path, &mut diagnostics) {
            for field in [
                "id",
                "version",
                "component",
                "permissions",
                "supported_contracts",
                "input_schema",
                "output_schema",
                "examples",
            ] {
                require_field(&json, &path, field, &mut diagnostics);
            }
            require_bool_value(
                &json,
                &path,
                "deterministic_default",
                true,
                &mut diagnostics,
            );
            let op_dir = path.parent().unwrap_or_else(|| Path::new("."));
            for relative in ["input.schema.json", "output.schema.json"] {
                let schema_path = op_dir.join(relative);
                if let Some(schema) = read_json(&schema_path, &mut diagnostics) {
                    require_bool_value(
                        &schema,
                        &schema_path,
                        "additionalProperties",
                        false,
                        &mut diagnostics,
                    );
                }
            }
            if !op_dir.join("examples/example.json").is_file() {
                diagnostics.push(Diagnostic::new(
                    "op.missing_example",
                    display_path(op_dir.join("examples/example.json")),
                    "examples",
                    "operation example is missing",
                ));
            }
        }
    }
    ValidationReport::from_diagnostics(diagnostics)
}

fn validate_flows(root: &Path) -> ValidationReport {
    validate_named_json_collection(root, "flows", "flow.json", &["id", "version", "steps"])
}

fn validate_views(root: &Path) -> ValidationReport {
    validate_named_json_collection(root, "views", "view.json", &["id", "version", "renderer"])
}

fn validate_playbooks(root: &Path) -> ValidationReport {
    let mut report = validate_named_json_collection(
        root,
        "playbooks",
        "playbook.json",
        &["id", "version", "sensor_requirements", "failure_modes"],
    );
    let mut diagnostics = Vec::new();
    for path in find_named_files(&root.join("playbooks"), "playbook.json") {
        if let Some(json) = read_json(&path, &mut diagnostics) {
            playbook_refs::check(&path, &json, &mut diagnostics);
            playbook_gate::check(&path, &json, &mut diagnostics);
        }
    }
    report.merge(ValidationReport::from_diagnostics(diagnostics));
    report
}

fn validate_named_json_collection(
    root: &Path,
    directory: &str,
    filename: &str,
    required: &[&str],
) -> ValidationReport {
    let mut diagnostics = Vec::new();
    for path in find_named_files(&root.join(directory), filename) {
        if let Some(json) = read_json(&path, &mut diagnostics) {
            for field in required {
                require_field(&json, &path, field, &mut diagnostics);
            }
            scan_banned_terms(&path, &json, &mut diagnostics);
        }
    }
    ValidationReport::from_diagnostics(diagnostics)
}

fn list_json_items(
    root: &Path,
    filename: &str,
    id_path: &[&str],
) -> Result<Vec<CatalogItem>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut items = Vec::new();
    for path in find_named_files(root, filename) {
        if let Some(json) = read_json(&path, &mut diagnostics) {
            match json_pointer(&json, id_path).and_then(Value::as_str) {
                Some(id) => items.push(CatalogItem {
                    id: id.to_string(),
                    path: display_path(&path),
                }),
                None => diagnostics.push(Diagnostic::new(
                    "catalog.missing_id",
                    display_path(&path),
                    id_path.join("."),
                    "catalog item id is missing or not a string",
                )),
            }
        }
    }
    if diagnostics.is_empty() {
        items.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(items)
    } else {
        Err(diagnostics)
    }
}

fn find_json_files(root: &Path) -> Vec<PathBuf> {
    find_files(root, |path| {
        path.extension().is_some_and(|ext| ext == "json")
    })
}

fn find_named_files(root: &Path, filename: &str) -> Vec<PathBuf> {
    find_files(root, |path| {
        path.file_name().is_some_and(|name| name == filename)
    })
}

fn find_files(root: &Path, predicate: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(root, &predicate, &mut files);
    files.sort();
    files
}

fn collect_files(root: &Path, predicate: &impl Fn(&Path) -> bool, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, predicate, files);
        } else if predicate(&path) {
            files.push(path);
        }
    }
}

fn immediate_dirs(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut dirs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
}

fn read_json(path: &Path, diagnostics: &mut Vec<Diagnostic>) -> Option<Value> {
    match fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(json) => Some(json),
            Err(error) => {
                diagnostics.push(Diagnostic::new(
                    "json.parse",
                    display_path(path),
                    "$",
                    format!("invalid json: {error}"),
                ));
                None
            }
        },
        Err(error) => {
            diagnostics.push(Diagnostic::new(
                "file.read",
                display_path(path),
                "$",
                format!("could not read file: {error}"),
            ));
            None
        }
    }
}

fn require_field(json: &Value, path: &Path, field: &str, diagnostics: &mut Vec<Diagnostic>) {
    if json.get(field).is_none() {
        diagnostics.push(Diagnostic::new(
            "json.missing_field",
            display_path(path),
            field,
            "required field is missing",
        ));
    }
}

fn require_path(json: &Value, path: &Path, field_path: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    if json_pointer(json, field_path).is_none() {
        diagnostics.push(Diagnostic::new(
            "json.missing_field",
            display_path(path),
            field_path.join("."),
            "required field is missing",
        ));
    }
}

fn require_string_value(
    json: &Value,
    path: &Path,
    field: &str,
    expected: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if json.get(field).and_then(Value::as_str) != Some(expected) {
        diagnostics.push(Diagnostic::new(
            "json.invalid_value",
            display_path(path),
            field,
            format!("expected string value {expected}"),
        ));
    }
}

fn require_bool_value(
    json: &Value,
    path: &Path,
    field: &str,
    expected: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if json.get(field).and_then(Value::as_bool) != Some(expected) {
        diagnostics.push(Diagnostic::new(
            "json.invalid_value",
            display_path(path),
            field,
            format!("expected boolean value {expected}"),
        ));
    }
}

fn scan_banned_terms(path: &Path, json: &Value, diagnostics: &mut Vec<Diagnostic>) {
    let content = json.to_string().to_lowercase();
    let banned = BTreeMap::from([
        ("koncar", "customer-specific name is not allowed"),
        ("raw customer", "raw customer data marker is not allowed"),
        ("customer data", "customer data marker is not allowed"),
    ]);
    for (term, message) in banned {
        if content.contains(term) {
            diagnostics.push(Diagnostic::new(
                "policy.customer_specific_content",
                display_path(path),
                "$",
                message,
            ));
        }
    }
}

fn pointer_or_self<'a>(json: &'a Value, field_path: &[&str]) -> &'a Value {
    json_pointer(json, field_path).unwrap_or(json)
}

fn json_pointer<'a>(json: &'a Value, field_path: &[&str]) -> Option<&'a Value> {
    let mut current = json;
    for field in field_path {
        current = current.get(*field)?;
    }
    Some(current)
}

fn display_path(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().into_owned()
}
