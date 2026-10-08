//! Keeps `ui/src/lib/generated/protocol.ts` in step with the Rust types.
//! `cargo test -p protocol` fails with a diff when the file is stale;
//! `UPDATE_TYPES=1 cargo test -p protocol` rewrites it.

use std::path::PathBuf;

use ts_rs::{Config, TS};

use crate::{
    Catalog, Command, CueInfo, EntryInfo, EventRecord, ImportOffer, LinkStatus, LinkView, Live,
    NoteMapping, Notice, NoticeLevel, Phase, SetlistInfo, SetlistTransferView, Setting, Settings,
    SettingsView, SongInfo, Transport, WireEvent,
};

const HEADER: &str =
    "// Generated from crates/protocol by `UPDATE_TYPES=1 cargo test -p protocol`. Do not edit.\n";

fn render() -> Result<String, ts_rs::ExportError> {
    let config = Config::default();
    let mut out = String::from(HEADER);
    for declaration in [
        Phase::export_to_string(&config)?,
        Command::export_to_string(&config)?,
        Transport::export_to_string(&config)?,
        Setting::export_to_string(&config)?,
        WireEvent::export_to_string(&config)?,
        EventRecord::export_to_string(&config)?,
        SongInfo::export_to_string(&config)?,
        CueInfo::export_to_string(&config)?,
        EntryInfo::export_to_string(&config)?,
        SetlistInfo::export_to_string(&config)?,
        Catalog::export_to_string(&config)?,
        Live::export_to_string(&config)?,
        LinkStatus::export_to_string(&config)?,
        LinkView::export_to_string(&config)?,
        NoticeLevel::export_to_string(&config)?,
        Notice::export_to_string(&config)?,
        Settings::export_to_string(&config)?,
        NoteMapping::export_to_string(&config)?,
        SettingsView::export_to_string(&config)?,
        ImportOffer::export_to_string(&config)?,
        SetlistTransferView::export_to_string(&config)?,
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
    let actual = std::fs::read_to_string(target())
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert_eq!(
        actual, expected,
        "ui/src/lib/generated/protocol.ts is stale: run `UPDATE_TYPES=1 cargo test -p protocol`"
    );
    Ok(())
}
