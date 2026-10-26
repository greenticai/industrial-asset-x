//! Embeds the repository's packaged playbooks and component manifests so the
//! catalog tool reports what ships, without a hand-kept list.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, file_name: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path().join(file_name))
        .filter(|path| path.is_file())
        .collect();
    found.sort();
    found
}

fn emit(out: &mut fs::File, name: &str, paths: &[PathBuf]) -> std::io::Result<()> {
    writeln!(out, "pub const {name}: &[&str] = &[")?;
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        writeln!(out, "    include_str!({:?}),", path.display().to_string())?;
    }
    writeln!(out, "];")
}

fn main() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let repo = manifest_dir.join("../..");
    let playbooks_dir = repo.join("playbooks");
    let components_dir = repo.join("components");
    println!("cargo:rerun-if-changed={}", playbooks_dir.display());
    println!("cargo:rerun-if-changed={}", components_dir.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());
    let mut out = fs::File::create(out_dir.join("embedded_data.rs"))?;
    emit(
        &mut out,
        "PLAYBOOKS",
        &collect(&playbooks_dir, "playbook.json"),
    )?;
    emit(
        &mut out,
        "COMPONENT_MANIFESTS",
        &collect(&components_dir, "component.manifest.json"),
    )
}
