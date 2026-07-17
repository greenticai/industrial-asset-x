use std::process::Command;

#[test]
fn cli_catalog_summary_outputs_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_industrial-asset-x"))
        .args(["catalog", "summary"])
        .output()
        .expect("cli should run");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"contracts\""));
    assert!(stdout.contains("\"components\""));
}

#[test]
fn cli_rejects_incomplete_store_package() {
    let temp_dir = std::env::temp_dir().join(format!(
        "industrial-asset-x-bad-store-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_industrial-asset-x"))
        .args(["store", "validate", temp_dir.to_str().unwrap()])
        .output()
        .expect("cli should run");

    let _ = std::fs::remove_dir_all(&temp_dir);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("store.missing_file"));
}
