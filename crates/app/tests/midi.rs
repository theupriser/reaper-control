//! MIDI notes become commands on the bus: mapping, channel filter, debounce, release, v1 import.

use std::sync::Arc;
use std::time::Duration;

use app::app_config::AppConfig;
use app::command_bus::CommandBus;
use app::config_repository::ConfigRepository;
use app::config_store::ConfigStore;
use app::event_bus::EventBus;
use app::fake_clock::FakeClock;
use app::fake_driver::FakeDriver;
use app::intent::Intent;
use app::intent_dispatcher::IntentDispatcher;
use app::legacy_config_file::LegacyConfigFile;
use app::midi_config::MidiConfig;
use app::midi_router::MidiRouter;
use app::queue_settings::QueueSettings;
use protocol::{Command, LinkView, Live, Phase};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn router(config: MidiConfig, playing: bool) -> (MidiRouter, Arc<FakeDriver>, Arc<FakeClock>) {
    let driver = Arc::new(FakeDriver::default());
    let clock = Arc::new(FakeClock::default());
    let events = Arc::new(EventBus::default());
    let bus = Arc::new(CommandBus::new(
        driver.clone(),
        events.clone(),
        clock.clone(),
        QueueSettings {
            repeat_window: Duration::ZERO,
            ..QueueSettings::default()
        },
    ));
    let phase = if playing { Phase::Playing } else { Phase::Idle };
    let intents = Arc::new(IntentDispatcher::new(bus, events.clone(), move || {
        LinkView {
            live: Some(Live {
                phase,
                current_song: Some(0),
                next_song: Some(1),
                ..Live::default()
            }),
            ..LinkView::default()
        }
    }));
    (
        MidiRouter::new(config, intents, events, clock.clone()),
        driver,
        clock,
    )
}

fn note_on(channel: u8, note: u8, velocity: u8) -> [u8; 3] {
    [0x90 | channel, note, velocity]
}

#[test]
fn a_mapped_note_sends_its_command_and_the_play_toggle_follows_the_state() {
    let (stopped, driver, _) = router(MidiConfig::default(), false);
    stopped.handle(&note_on(0, 51, 100));
    stopped.handle(&note_on(0, 50, 100));
    let (playing, running, _) = router(MidiConfig::default(), true);
    playing.handle(&note_on(3, 50, 100));
    assert_eq!(driver.sent(), vec![Command::Next, Command::Play]);
    assert_eq!(running.sent(), vec![Command::Pause]);
}

#[test]
fn releases_other_channels_unmapped_notes_and_other_messages_do_nothing() {
    let config = MidiConfig {
        channel: Some(2),
        ..MidiConfig::default()
    };
    let (router, driver, _) = router(config, false);
    router.handle(&note_on(2, 51, 0));
    router.handle(&note_on(1, 51, 100));
    router.handle(&note_on(2, 99, 100));
    router.handle(&[0xB2, 51, 100]);
    router.handle(&[0x92]);
    assert_eq!(driver.sent(), Vec::<Command>::new());
    router.handle(&note_on(2, 51, 100));
    assert_eq!(driver.sent(), vec![Command::Next]);
}

#[test]
fn the_same_note_inside_the_window_counts_once() {
    let (router, driver, clock) = router(MidiConfig::default(), false);
    router.handle(&note_on(0, 51, 100));
    clock.advance(Duration::from_millis(150));
    router.handle(&note_on(0, 51, 100));
    router.handle(&note_on(0, 48, 100));
    clock.advance(Duration::from_millis(100));
    router.handle(&note_on(0, 51, 100));
    assert_eq!(
        driver.sent(),
        vec![Command::Next, Command::Previous, Command::Next]
    );
}

#[test]
fn the_v1_config_gives_the_mapping_and_names_what_has_no_counterpart() -> TestResult {
    let directory = std::env::temp_dir().join(format!("app-midi-legacy-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let file = directory.join("config.json");
    std::fs::write(
        &file,
        r#"{"reaper":{"host":"127.0.0.1"},"midi":{"enabled":true,"deviceName":"Pedal","channel":4,
        "noteMapping":{"60":"nextRegion","61":"togglePlay","62":"somethingNew","200":"pause","63":7}}}"#,
    )?;
    let (config, skipped) = LegacyConfigFile::read(&file)?.midi.into_config();
    assert_eq!(config.device_name.as_deref(), Some("Pedal"));
    assert_eq!(config.channel, Some(4));
    assert_eq!(config.notes.get(&60), Some(&Intent::Next));
    assert_eq!(config.notes.get(&61), Some(&Intent::TogglePlay));
    assert_eq!(config.notes.get(&44), Some(&Intent::RestartSong));
    assert_eq!(skipped, vec!["200: pause", "62: somethingNew", "63: ?"]);
    let odd = directory.join("odd.json");
    std::fs::write(
        &odd,
        r#"{"midi":{"channel":-1,"noteMapping":{"60":"pause"}}}"#,
    )?;
    let (config, skipped) = LegacyConfigFile::read(&odd)?.midi.into_config();
    assert_eq!(config.channel, None);
    assert_eq!(skipped, vec!["channel: -1"]);
    assert_eq!(config.notes.get(&60), Some(&Intent::Pause));

    let store = ConfigStore::new(directory.join("saved.json"));
    let mut saved = AppConfig {
        midi: config,
        ..AppConfig::default()
    };
    store.save(&saved)?;
    assert_eq!(store.load()?, saved);
    saved.midi.channel = Some(16);
    assert!(store.save(&saved).is_err());
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

#[cfg(target_os = "macos")]
#[test]
fn a_note_from_a_real_midi_port_reaches_the_bus() -> TestResult {
    use app::midi_listener::MidiListener;
    use midir::MidiOutput;
    use midir::os::unix::VirtualOutput;

    let name = format!("RC2 test source {}", std::process::id());
    let output = MidiOutput::new("Reaper Control test")?;
    let mut source = output
        .create_virtual(&name)
        .map_err(|error| error.to_string())?;
    let (router, driver, _) = router(MidiConfig::default(), false);
    MidiListener::start(
        Arc::new(app::midir_source::MidirSource),
        Arc::new(router),
        Some(name),
        Arc::default(),
    )?;
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while driver.sent().is_empty() && std::time::Instant::now() < deadline {
        source.send(&note_on(0, 51, 100))?;
        std::thread::sleep(Duration::from_millis(300));
    }
    assert_eq!(driver.sent().first(), Some(&Command::Next));
    Ok(())
}

type Changes = Arc<std::sync::Mutex<Vec<Vec<String>>>>;

fn follower(
    source: &Arc<app::fake_midi_source::FakeMidiSource>,
    wanted: Option<&str>,
) -> (
    app::midi_device_follower::MidiDeviceFollower,
    Arc<FakeDriver>,
    Changes,
) {
    let (router, driver, _) = router(MidiConfig::default(), false);
    let events = Arc::new(EventBus::default());
    let changes = Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = Arc::clone(&changes);
    events.subscribe(move |event| {
        if let app::app_event::AppEvent::MidiDevicesChanged { devices } = event
            && let Ok(mut seen) = seen.lock()
        {
            seen.push(devices.clone());
        }
    });
    let follower = app::midi_device_follower::MidiDeviceFollower::new(
        source.clone(),
        Arc::new(router),
        wanted.map(str::to_owned),
        events,
    );
    (follower, driver, changes)
}

#[test]
fn a_plugged_in_device_is_opened_once_and_its_notes_reach_the_bus() {
    let source = Arc::new(app::fake_midi_source::FakeMidiSource::default());
    let (mut follower, driver, changes) = follower(&source, None);
    source.plug(Some(&["FootCtrl Mini"]));
    follower.poll();
    follower.poll();
    assert_eq!(source.open_devices(), vec!["FootCtrl Mini".to_owned()]);
    assert_eq!(changes.lock().map(|c| c.len()).unwrap_or_default(), 1);
    assert!(source.send("FootCtrl Mini", &note_on(0, 51, 100)));
    assert_eq!(driver.sent().first(), Some(&Command::Next));
}

#[test]
fn an_unplugged_device_is_closed_and_a_failed_listing_closes_nothing() {
    let source = Arc::new(app::fake_midi_source::FakeMidiSource::default());
    let (mut follower, _, changes) = follower(&source, None);
    source.plug(Some(&["A", "B"]));
    follower.poll();
    source.plug(None);
    follower.poll();
    assert_eq!(source.open_devices(), vec!["A".to_owned(), "B".to_owned()]);
    source.plug(Some(&["B"]));
    follower.poll();
    assert_eq!(source.open_devices(), vec!["B".to_owned()]);
    assert!(!source.send("A", &note_on(0, 51, 100)));
    let changes = changes.lock().map(|c| c.clone()).unwrap_or_default();
    assert_eq!(
        changes,
        vec![vec!["A".to_owned(), "B".to_owned()], vec!["B".to_owned()]]
    );
}

#[test]
fn only_the_named_device_is_opened_when_one_is_named() {
    let source = Arc::new(app::fake_midi_source::FakeMidiSource::default());
    let (mut follower, _, _) = follower(&source, Some("B"));
    source.plug(Some(&["A", "B"]));
    follower.poll();
    assert_eq!(source.open_devices(), vec!["B".to_owned()]);
}
