//! Size audit (PLAN WP 4b.1): the size of every public type, so growth is seen and the rule
//! "under 128 bytes, ideally 64 or less" (AGENTS.md "Memory and layout") can be checked.
//!
//! `cargo test -p app --test size_audit -- --nocapture` prints the table.
//! The budget (AGENTS.md): at most 128 bytes, ideally 64. `Catalog` is the one exception, a bag of
//! lists that is only ever held behind a `Box`; the hot types are named in `HOT_TYPES`.
//! Generic types and the extension's types are not listed.

use std::mem::size_of;
use std::path::Path;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Over the 128 byte budget on purpose, with the reason.
const ALLOWED_OVER_BUDGET: &[&str] = &[
    // Five lists and a string; always boxed in messages, events and views.
    "protocol::Catalog",
];

/// Types that travel through queues and channels; named so a failure says which one grew.
const HOT_TYPES: &[&str] = &[
    "protocol::Command",
    "app::AppEvent",
    "performance::Input",
    "performance::Effect",
    "protocol::Live",
    "protocol::message::ClientMessage",
    "protocol::message::ServerMessage",
    "link::LinkEvent",
];

macro_rules! sizes {
    ($($path:path),* $(,)?) => {
        vec![$((stringify!($path).replace(" ", ""), size_of::<$path>())),*]
    };
}

fn all_sizes() -> Vec<(String, usize)> {
    let mut sizes = sizes![
        app::AppConfig,
        app::MidiListener,
        app::MidiSwitch,
        app::BinaryArchitecture,
        app::ExtensionInstaller,
        app::InstallAction,
        app::InstallError,
        app::InstallItem,
        app::InstallItemKind,
        app::InstallPlan,
        app::InstallPlatform,
        app::InstallReport,
        app::InstallStatus,
        app::InstallVerification,
        app::MacExtensionPreparer,
        app::PlainExtensionPreparer,
        app::ReaperInstall,
        app::AppEvent,
        app::AppFault,
        app::ChaosProxy,
        app::ChaosSettings,
        app::CommandBus,
        app::CommandQueue,
        app::CommandRefusal,
        app::ConfigError,
        app::ConfigStore,
        app::DiagnosticsBundle,
        app::DiagnosticsError,
        app::DispatchError,
        app::DriverError,
        app::EventBus,
        app::FakeClock,
        app::FakeConfigRepository,
        app::FakeDriver,
        app::FakeExtension,
        app::FakeFaultCheck,
        app::FakeMidiSource,
        app::FakeMirrorRepository,
        app::FakeProcessCheck,
        app::FakeStatsSource,
        app::FaultFileCheck,
        app::HealthMonitor,
        app::ImportError,
        app::IntentDispatcher,
        app::IntentError,
        app::IntentRefusal,
        app::IntentTranslator,
        app::Intent,
        app::JournalImport,
        app::LatencyStats,
        app::LegacyConfigFile,
        app::LegacyFile,
        app::LegacyItem,
        app::LegacyMidi,
        app::LegacySetlist,
        app::LinkCause,
        app::LinkConnection,
        app::LinkHealth,
        app::LinkPipeline,
        app::LinkSession,
        app::LogConfig,
        app::Logging,
        app::MessageTranslator,
        app::MeteredDriver,
        app::MidiConfig,
        app::MidiDebounce,
        app::MidiDeviceFollower,
        app::MidiListener,
        app::MidiMessage,
        app::MidiRouter,
        app::MidirSource,
        app::MirrorError,
        app::MirrorFile,
        app::MirrorKeeper,
        app::PendingCommand,
        app::PeriodicThread,
        app::QueueConfig,
        app::QueueRejection,
        app::QueueSettings,
        app::ResolvedSetlist,
        app::SetlistMirror,
        app::SetlistTransfer,
        app::SettingsService,
        app::ShutdownSequence,
        app::Simulator,
        app::SysinfoStatsSource,
        app::SystemClock,
        app::SystemProcessCheck,
        app::SystemStatsService,
        catalogue::Cue,
        catalogue::Directive,
        catalogue::Directives,
        catalogue::InvalidSong,
        catalogue::MarkerName,
        catalogue::Song,
        link::LinkClient,
        link::ClientConfig,
        link::LinkEvent,
        link::SendError,
        link::Endpoint,
        link::EndpointError,
        link::EventLog,
        link::Replay,
        link::LinkServer,
        link::ServerError,
        performance::Effect,
        performance::Event,
        performance::Flag,
        performance::Flags,
        performance::HandOverPolicy,
        performance::Input,
        performance::InvalidTempoMap,
        performance::Output,
        performance::Performance,
        performance::Phase,
        performance::PlannedSong,
        performance::Rejection,
        performance::SongWindow,
        performance::TempoMap,
        performance::TempoSegment,
        performance::TimeSignature,
        projections::Applied,
        projections::EntryView,
        projections::Freshness,
        projections::LiveFeed,
        projections::PerformanceView,
        projections::Plan,
        projections::PlayerView,
        projections::SetlistView,
        protocol::Catalog,
        protocol::Command,
        protocol::CueInfo,
        protocol::EntryInfo,
        protocol::EventRecord,
        protocol::frame::FrameDecoder,
        protocol::frame::FrameError,
        protocol::ImportOffer,
        protocol::LinkStatus,
        protocol::LinkView,
        protocol::Live,
        protocol::message::ClientMessage,
        protocol::message::CodecError,
        protocol::message::HandshakeError,
        protocol::message::Outcome,
        protocol::message::ServerMessage,
        protocol::ActionChoice,
        protocol::NoteMapping,
        protocol::NoticeLevel,
        protocol::Notice,
        protocol::Phase,
        protocol::SetlistInfo,
        protocol::SetlistTransferView,
        protocol::Setting,
        protocol::SettingsView,
        protocol::Settings,
        protocol::SongInfo,
        protocol::SystemStats,
        protocol::Transport,
        protocol::WireEvent,
        reaper_port::FakeReaper,
        reaper_port::Marker,
        reaper_port::Region,
        reaper_port::Scenario,
        reaper_port::ScenarioError,
        reaper_port::Expectation,
        reaper_port::ScenarioRunner,
        reaper_port::ScenarioSong,
        reaper_port::Step,
        reaper_port::Trace,
        reaper_port::Transport,
        setlists::Edit,
        setlists::EntryId,
        setlists::Entry,
        setlists::InMemorySetlistRepository,
        setlists::InvalidSetlist,
        setlists::Rejection,
        setlists::Revision,
        setlists::SaveError,
        setlists::SetlistEvent,
        setlists::SetlistId,
        setlists::Setlist,
        shared_kernel::Bpm,
        shared_kernel::InvalidValue,
        shared_kernel::Seconds,
        shared_kernel::SongId,
        timer_loop::QueuedCommands,
        timer_loop::StatePublisher,
    ];
    sizes.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    sizes
}

#[test]
fn print_the_size_of_every_public_type() {
    let sizes = all_sizes();
    println!("{:>6}  type", "bytes");
    for (name, size) in &sizes {
        println!("{size:>6}  {name}");
    }
    let over_64 = sizes.iter().filter(|(_, size)| *size > 64).count();
    let over_128 = sizes.iter().filter(|(_, size)| *size > 128).count();
    println!(
        "{} types, {over_64} over 64 bytes, {over_128} over 128 bytes",
        sizes.len()
    );
}

/// The names of the non-generic public structs and enums declared in `directory`.
fn declared_types(directory: &Path, found: &mut Vec<String>) -> TestResult {
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            declared_types(&path, found)?;
        } else if path.extension().is_some_and(|extension| extension == "rs")
            && path.file_name().is_some_and(|name| name != "tests.rs")
        {
            for line in std::fs::read_to_string(&path)?.lines() {
                let declaration = line
                    .strip_prefix("pub struct ")
                    .or_else(|| line.strip_prefix("pub enum "));
                let name: String = declaration
                    .map(|rest| {
                        rest.chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_')
                            .collect()
                    })
                    .unwrap_or_default();
                let generic = declaration.is_some_and(|rest| rest[name.len()..].starts_with('<'));
                if !name.is_empty() && !generic {
                    found.push(name);
                }
            }
        }
    }
    Ok(())
}

#[test]
fn every_public_type_is_in_the_table() -> TestResult {
    let listed: Vec<String> = all_sizes()
        .into_iter()
        .map(|(path, _)| path.rsplit("::").next().unwrap_or_default().to_string())
        .collect();
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(crates)? {
        let crate_directory = entry?.path();
        let name = crate_directory
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if matches!(name, "architecture-tests" | "reaper-extension")
            || !crate_directory.join("src").is_dir()
        {
            continue;
        }
        let mut found = Vec::new();
        declared_types(&crate_directory.join("src"), &mut found)?;
        missing.extend(
            found
                .into_iter()
                .filter(|type_name| !listed.contains(type_name))
                .map(|type_name| format!("{name}: {type_name}")),
        );
    }
    assert!(
        missing.is_empty(),
        "add these to the size table:\n{}",
        missing.join("\n")
    );
    Ok(())
}

#[test]
fn no_type_is_over_128_bytes_unless_allowed() {
    let too_big: Vec<String> = all_sizes()
        .into_iter()
        .filter(|(name, size)| *size > 128 && !ALLOWED_OVER_BUDGET.contains(&name.as_str()))
        .map(|(name, size)| format!("{name}: {size} bytes"))
        .collect();
    assert!(
        too_big.is_empty(),
        "over 128 bytes, box the heavy part with a one-line comment:\n{}",
        too_big.join("\n")
    );
}

#[test]
fn the_hot_types_stay_within_the_budget() {
    let sizes = all_sizes();
    for name in HOT_TYPES {
        let size = sizes
            .iter()
            .find(|(listed, _)| listed == name)
            .map(|(_, size)| *size);
        assert!(
            size.is_some_and(|size| size <= 96),
            "{name} is {size:?} bytes; hot types stay at 96 or less"
        );
    }
}
