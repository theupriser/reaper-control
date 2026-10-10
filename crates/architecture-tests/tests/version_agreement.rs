//! The app and the extension have their own versions (owner's decision, 2026-10-10: app 2.0.0,
//! extension 1.0.0, then 1.0.1 for the hard stop fix). Every crate takes the app version except the extension, whose version is
//! the one the installer's handshake check (SPEC §13.1 step 6) expects (WP 3.10).

use std::fs;
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn workspace_version() -> Result<String, Box<dyn std::error::Error>> {
    let manifest = fs::read_to_string(root().join("Cargo.toml"))?;
    manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| "no version in the workspace Cargo.toml".into())
}

#[test]
fn every_crate_takes_the_workspace_version() -> TestResult {
    for entry in fs::read_dir(root().join("crates"))? {
        let directory = entry?.path();
        if directory.ends_with("reaper-extension") {
            continue;
        }
        let manifest = directory.join("Cargo.toml");
        let text = fs::read_to_string(&manifest)?;
        assert!(
            text.lines()
                .any(|line| line.trim() == "version.workspace = true"),
            "{} has its own version; use version.workspace = true",
            manifest.display()
        );
    }
    Ok(())
}

#[test]
fn the_extension_has_its_own_version() -> TestResult {
    let text = fs::read_to_string(root().join("crates/reaper-extension/Cargo.toml"))?;
    assert!(
        text.lines()
            .any(|line| line.trim() == "version = \"1.0.1\""),
        "the extension version is 1.0.1 until the extension changes"
    );
    Ok(())
}

#[test]
fn the_app_expects_the_version_the_extension_has() -> TestResult {
    let manifest = fs::read_to_string(root().join("crates/reaper-extension/Cargo.toml"))?;
    let version = manifest
        .lines()
        .find_map(|line| line.trim().strip_prefix("version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .ok_or("no version in the extension manifest")?;
    let shell = fs::read_to_string(root().join("crates/app/src/shell.rs"))?;
    assert!(
        shell.contains(&format!("BUNDLED_EXTENSION_VERSION: &str = \"{version}\"")),
        "shell.rs must expect extension version {version}"
    );
    Ok(())
}

#[test]
fn the_app_window_and_the_ui_package_carry_the_same_version() -> TestResult {
    let version = workspace_version()?;
    for file in ["crates/app/tauri.conf.json", "ui/package.json"] {
        let text = fs::read_to_string(root().join(file))?;
        let json: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(found) = json.get("version").and_then(|value| value.as_str()) {
            assert_eq!(found, version, "{file} disagrees with Cargo.toml");
        }
    }
    Ok(())
}

#[test]
fn the_release_workflow_checks_the_tag_against_the_version() -> TestResult {
    let workflow = fs::read_to_string(root().join(".github/workflows/release.yml"))?;
    assert!(workflow.contains("tags: [\"v*\"]"));
    assert!(
        workflow.contains("--draft"),
        "a release must open as a draft"
    );
    Ok(())
}
