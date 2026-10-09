//! The ubiquitous language (SPEC §14.2): words the glossary bans must not appear in the code
//! or the UI copy. "transition" is "hand-over" and "playlist" is "setlist".
//!
//! Scanned: every Rust file under `crates/*/src` and `crates/*/tests` (this crate excluded) and
//! the UI sources, minus the generated types. CSS in a `<style>` block is not copy and is skipped.

use std::path::{Path, PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const BANNED: &[(&str, &str)] = &[("transition", "hand-over"), ("playlist", "setlist")];

/// Lines of `text` outside `<style>` blocks that contain a banned word, as (line number, word, use instead).
fn banned_uses(text: &str) -> Vec<(usize, &'static str, &'static str)> {
    let mut found = Vec::new();
    let mut in_style = false;
    for (index, line) in text.lines().enumerate() {
        if line.contains("<style") {
            in_style = true;
        }
        if !in_style {
            let lower = line.to_lowercase();
            for (word, instead) in BANNED {
                if lower.contains(word) {
                    found.push((index + 1, *word, *instead));
                }
            }
        }
        if line.contains("</style>") {
            in_style = false;
        }
    }
    found
}

fn source_files(directory: &Path, extensions: &[&str], found: &mut Vec<PathBuf>) -> TestResult {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if !matches!(name, "node_modules" | "generated" | "target" | "dist") {
                source_files(&path, extensions, found)?;
            }
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extensions.contains(&extension))
        {
            found.push(path);
        }
    }
    Ok(())
}

#[test]
fn the_scan_finds_a_banned_word_and_skips_css() {
    let text = "let a = 1;\nfn smooth_transition() {}\n<style>\n.bar { transition: width 1s; }\n</style>\nPlaylist";
    let uses = banned_uses(text);
    assert_eq!(
        uses,
        vec![(2, "transition", "hand-over"), (6, "playlist", "setlist")]
    );
}

#[test]
fn no_banned_word_is_used_in_the_code_or_the_copy() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root.join("crates"))? {
        let crate_directory = entry?.path();
        if crate_directory.ends_with("architecture-tests") {
            continue;
        }
        for part in ["src", "tests"] {
            if crate_directory.join(part).is_dir() {
                source_files(&crate_directory.join(part), &["rs"], &mut files)?;
            }
        }
    }
    source_files(&root.join("ui/src"), &["ts", "svelte"], &mut files)?;

    let mut problems = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file)?;
        for (line, word, instead) in banned_uses(&text) {
            problems.push(format!(
                "{}:{line}: \"{word}\", say \"{instead}\" (SPEC §14.2)",
                file.strip_prefix(&root).unwrap_or(&file).display()
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "banned words:\n{}",
        problems.join("\n")
    );
    Ok(())
}
