//! Keeps `ui/src/lib/generated/protocol.ts` in step with the Rust types.
//! `cargo test -p protocol` fails with a diff when the file is stale;
//! `UPDATE_TYPES=1 cargo test -p protocol` rewrites it.

use std::path::PathBuf;

use ts_rs::{Config, TS};

use crate::{AppState, Command, Phase};

const HEADER: &str =
    "// Generated from crates/protocol by `UPDATE_TYPES=1 cargo test -p protocol`. Do not edit.\n";

fn render() -> Result<String, ts_rs::ExportError> {
    let config = Config::default();
    let mut out = String::from(HEADER);
    for declaration in [
        Phase::export_to_string(&config)?,
        Command::export_to_string(&config)?,
        AppState::export_to_string(&config)?,
    ] {
        out.push('\n');
        out.push_str(&strip_imports(&declaration));
    }
    Ok(out)
}

fn strip_imports(declaration: &str) -> String {
    declaration
        .lines()
        .filter(|l| !l.starts_with("// This file was generated") && !l.starts_with("import "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
        + "\n"
}

fn target() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui/src/lib/generated/protocol.ts")
}

#[test]
fn typescript_matches_the_rust_types() -> Result<(), Box<dyn std::error::Error>> {
    let expected = render()?;
    if std::env::var_os("UPDATE_TYPES").is_some() {
        std::fs::write(target(), &expected)?;
        return Ok(());
    }
    let actual = std::fs::read_to_string(target()).unwrap_or_default();
    assert_eq!(
        actual, expected,
        "ui/src/lib/generated/protocol.ts is stale: run `UPDATE_TYPES=1 cargo test -p protocol`"
    );
    Ok(())
}
