//! File and module layout (AGENTS.md "Memory and layout", PLAN WP 4b.4): no `mod.rs`, names are
//! `lower_snake_case`, a module with submodules is `x.rs` next to `x/`, plural names are listed,
//! and the only public modules are the named groups below.

use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Files and folders that end in "s" on purpose: they are named after the type, function or
/// constant they hold (`Directives`, `plan_songs`, `Seconds`), or they hold test cases.
const PLURAL_ALLOWED: &[&str] = &[
    "chaos_settings",
    "command_bus",
    "directives",
    "event_bus",
    "flags",
    "freshness",
    "incoming_commands",
    "install_status",
    "latency_stats",
    "link_status",
    "missing_functions",
    "performance_properties",
    "performance_runs",
    "plan_songs",
    "project_setlists",
    "project_switch_tests",
    "properties",
    "queue_settings",
    "queued_commands",
    "recent_answers",
    "required_functions",
    "seconds",
    "setlist_properties",
    "settings",
    "system_stats",
    "tempo_map_tests",
    "tests",
    "wizard_step_status",
];

/// `pub mod` lines that are allowed, as (crate, module): named groups whose items keep their group.
const PUBLIC_MODULES: &[(&str, &str)] = &[("protocol", "frame"), ("protocol", "message")];

fn crates() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.join("src").is_dir() {
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}

/// Every file and folder below `directory`, skipping build output and the fuzz harness.
fn entries(directory: &Path, found: &mut Vec<PathBuf>) -> TestResult {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if matches!(name, "target" | "fuzz" | "node_modules") {
            continue;
        }
        found.push(path.clone());
        if path.is_dir() {
            entries(&path, found)?;
        }
    }
    Ok(())
}

fn is_lower_snake_case(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with('_')
        && !name.contains("__")
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string()
}

fn module_entries(crate_directory: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut found = Vec::new();
    for part in ["src", "tests"] {
        if crate_directory.join(part).is_dir() {
            entries(&crate_directory.join(part), &mut found)?;
        }
    }
    Ok(found)
}

#[test]
fn there_is_no_mod_rs() -> TestResult {
    let mut problems = Vec::new();
    for crate_directory in crates()? {
        for path in module_entries(&crate_directory)? {
            if path.file_name().is_some_and(|name| name == "mod.rs") {
                problems.push(path.display().to_string());
            }
        }
    }
    assert!(
        problems.is_empty(),
        "use `x.rs` next to `x/`, never `mod.rs`:\n{}",
        problems.join("\n")
    );
    Ok(())
}

#[test]
fn files_and_folders_are_lower_snake_case() -> TestResult {
    let mut problems = Vec::new();
    for crate_directory in crates()? {
        for path in module_entries(&crate_directory)? {
            let is_rust = path.extension().is_some_and(|extension| extension == "rs");
            if (is_rust || path.is_dir()) && !is_lower_snake_case(&stem(&path)) {
                problems.push(path.display().to_string());
            }
        }
    }
    assert!(
        problems.is_empty(),
        "not lower_snake_case:\n{}",
        problems.join("\n")
    );
    Ok(())
}

#[test]
fn a_module_with_submodules_is_a_file_next_to_a_folder() -> TestResult {
    let mut problems = Vec::new();
    for crate_directory in crates()? {
        let source = crate_directory.join("src");
        for path in module_entries(&crate_directory)? {
            let inside_source = path.starts_with(&source) && path != source;
            if inside_source && path.is_dir() {
                let sibling = path.with_extension("rs");
                if !sibling.is_file() {
                    problems.push(format!("{} has no {}", path.display(), sibling.display()));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "a folder needs its `x.rs`:\n{}",
        problems.join("\n")
    );
    Ok(())
}

#[test]
fn plural_names_are_on_the_list() -> TestResult {
    let mut problems = Vec::new();
    for crate_directory in crates()? {
        let source = crate_directory.join("src");
        for path in module_entries(&crate_directory)? {
            let is_module =
                path.is_dir() || path.extension().is_some_and(|extension| extension == "rs");
            let name = stem(&path);
            if path.starts_with(&source)
                && path != source
                && is_module
                && name.ends_with('s')
                && !PLURAL_ALLOWED.contains(&name.as_str())
            {
                problems.push(path.display().to_string());
            }
        }
    }
    assert!(
        problems.is_empty(),
        "module names are singular; rename, or add to PLURAL_ALLOWED with the type it is named after:\n{}",
        problems.join("\n")
    );
    Ok(())
}

#[test]
fn only_the_named_groups_are_public_modules() -> TestResult {
    let mut problems = Vec::new();
    for crate_directory in crates()? {
        let crate_name = crate_directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        for path in module_entries(&crate_directory)? {
            if !path.starts_with(crate_directory.join("src"))
                || path.extension().is_none_or(|extension| extension != "rs")
            {
                continue;
            }
            for line in std::fs::read_to_string(&path)?.lines() {
                let Some(rest) = line.strip_prefix("pub mod ") else {
                    continue;
                };
                let module = rest.trim_end_matches(';').trim();
                if !PUBLIC_MODULES.contains(&(crate_name.as_str(), module)) {
                    problems.push(format!("{}: pub mod {module}", path.display()));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "keep modules private and `pub use` the API:\n{}",
        problems.join("\n")
    );
    Ok(())
}
