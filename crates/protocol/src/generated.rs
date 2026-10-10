//! Keeps `ui/src/lib/generated/protocol.ts` in step with the Rust types.
//! `cargo test -p protocol` fails with a diff when the file is stale;
//! `UPDATE_TYPES=1 cargo test -p protocol` rewrites it.

use std::path::PathBuf;

use ts_rs::{Config, TS};

use crate::{
    ActionChoice, AppearanceChoice, Catalog, CheckId, CheckItem, CheckStatus, ChecklistView,
    Command, CueInfo, EntryInfo, EventRecord, ImportOffer, InstallationView, LinkProblem,
    LinkStatus, LinkView, Live, NoteMapping, Notice, NoticeLevel, Phase, SetlistInfo,
    SetlistTransferView, Setting, Settings, SettingsView, SongInfo, SystemStats, Transport,
    WireEvent, WizardStep, WizardStepId, WizardStepStatus,
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
        LinkProblem::export_to_string(&config)?,
        Settings::export_to_string(&config)?,
        NoteMapping::export_to_string(&config)?,
        ActionChoice::export_to_string(&config)?,
        AppearanceChoice::export_to_string(&config)?,
        SettingsView::export_to_string(&config)?,
        ImportOffer::export_to_string(&config)?,
        SetlistTransferView::export_to_string(&config)?,
        SystemStats::export_to_string(&config)?,
        WizardStepId::export_to_string(&config)?,
        WizardStepStatus::export_to_string(&config)?,
        WizardStep::export_to_string(&config)?,
        InstallationView::export_to_string(&config)?,
        CheckId::export_to_string(&config)?,
        CheckStatus::export_to_string(&config)?,
        CheckItem::export_to_string(&config)?,
        ChecklistView::export_to_string(&config)?,
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

/// The version and the fingerprint of the types the extension link carries. The extension and the
/// app are built apart, so a change to these types without a new `PROTOCOL_VERSION` lets an old
/// extension connect and then fail on a message it cannot read.
const LINK_SCHEMA: (u32, u64) = (3, 3799876410148336785);

fn fingerprint(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[test]
fn a_changed_link_protocol_needs_a_new_version() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let mut text = String::new();
    for declaration in [
        Command::export_to_string(&config)?,
        WireEvent::export_to_string(&config)?,
        EventRecord::export_to_string(&config)?,
        Live::export_to_string(&config)?,
        Catalog::export_to_string(&config)?,
        SongInfo::export_to_string(&config)?,
        CueInfo::export_to_string(&config)?,
        SetlistInfo::export_to_string(&config)?,
        EntryInfo::export_to_string(&config)?,
        Phase::export_to_string(&config)?,
        Transport::export_to_string(&config)?,
        Setting::export_to_string(&config)?,
    ] {
        for line in strip_imports(&declaration).lines() {
            let line = line.trim();
            if !(line.starts_with("/**") || line.starts_with('*') || line.starts_with("//")) {
                text.push_str(line);
            }
        }
    }
    let found = (
        crate::message::PROTOCOL_VERSION,
        fingerprint(&text.replace("\r\n", "\n")),
    );
    assert_eq!(
        found, LINK_SCHEMA,
        "the link types changed: raise PROTOCOL_VERSION if an older extension cannot read them, then set LINK_SCHEMA to {found:?}"
    );
    Ok(())
}
