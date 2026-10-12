//! The `testing/` folder has a fixed shape (testing/README.md): fixtures in `data/`, scripts in
//! `scripts/`, and nothing loose next to them.

use std::fs;
use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn testing() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testing")
}

fn names(directory: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(directory)? {
        names.push(entry?.file_name().to_string_lossy().into_owned());
    }
    names.sort();
    Ok(names)
}

#[test]
fn testing_holds_only_the_readme_data_and_scripts() -> TestResult {
    for name in names(&testing())? {
        assert!(
            ["README.md", "data", "scripts"].contains(&name.as_str()),
            "testing/{name} does not belong here; see testing/README.md"
        );
    }
    Ok(())
}

#[test]
fn data_holds_only_projects_reaper_scenarios_and_vectors() -> TestResult {
    assert_eq!(
        names(&testing().join("data"))?,
        ["projects", "reaper", "scenarios", "vectors"]
    );
    Ok(())
}

#[test]
fn scripts_are_shell_scripts_with_a_header_and_one_group_folder() -> TestResult {
    let scripts = testing().join("scripts");
    let mut files = Vec::new();
    for name in names(&scripts)? {
        if name == "chaos" {
            for inner in names(&scripts.join("chaos"))? {
                files.push(scripts.join("chaos").join(inner));
            }
        } else {
            files.push(scripts.join(name));
        }
    }
    for file in files {
        let text = fs::read_to_string(&file)?;
        let mut lines = text.lines();
        assert_eq!(
            lines.next(),
            Some("#!/usr/bin/env bash"),
            "{} must start with the bash shebang",
            file.display()
        );
        assert!(
            lines.next().is_some_and(|line| line.starts_with("# ")),
            "{} needs a header comment saying what it checks",
            file.display()
        );
    }
    Ok(())
}
