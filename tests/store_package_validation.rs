use std::path::Path;

use industrial_asset_x::{validate_catalog, validate_store_package};

#[test]
fn transformer_store_package_is_complete() {
    let report = validate_store_package(Path::new(
        "store/examples/substation-transformer-health-basic",
    ));
    assert!(report.valid, "{:?}", report.diagnostics);
}

#[test]
fn catalog_validation_passes() {
    let report = validate_catalog(Path::new("."));
    assert!(report.valid, "{:?}", report.diagnostics);
}
