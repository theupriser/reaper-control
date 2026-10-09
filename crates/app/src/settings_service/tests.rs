use std::time::Duration;

use protocol::Command;

use super::*;
use crate::config_store::ConfigStore;
use crate::event_bus::EventBus;
use crate::fake_clock::FakeClock;
use crate::fake_driver::FakeDriver;
use crate::queue_settings::QueueSettings;

struct Rig {
    directory: std::path::PathBuf,
    driver: Arc<FakeDriver>,
    clock: Arc<FakeClock>,
    bus: Arc<CommandBus>,
    service: SettingsService,
}

fn rig(name: &str) -> Rig {
    let directory =
        std::env::temp_dir().join(format!("rc2-settings-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    let driver = Arc::new(FakeDriver::default());
    let clock = Arc::new(FakeClock::default());
    let bus = Arc::new(CommandBus::new(
        driver.clone(),
        Arc::new(EventBus::default()),
        clock.clone(),
        QueueSettings::default(),
    ));
    let service = SettingsService::new(
        Arc::new(ConfigStore::new(directory.join("config.json"))),
        AppConfig::default(),
        bus.clone(),
        Arc::new(|_| {}),
    );
    Rig {
        directory,
        driver,
        clock,
        bus,
        service,
    }
}

#[test]
fn the_view_shows_the_defaults_the_devices_and_the_note_table() {
    let rig = rig("view");
    let view = rig.service.view(vec!["FootCtrl Mini".into()]);
    assert_eq!(view.settings, AppConfig::default().settings());
    assert_eq!(view.devices, vec!["FootCtrl Mini".to_owned()]);
    assert_eq!(view.notes.len(), 8);
    assert_eq!(
        view.notes.first().map(|n| (n.note, n.action.as_str())),
        Some((44, "Restart song"))
    );
}

#[test]
fn a_saved_value_is_on_disk_and_loads_back() -> Result<(), ConfigError> {
    let rig = rig("save");
    let mut settings = rig.service.view(Vec::new()).settings;
    settings.midi_channel = Some(3);
    settings.midi_device_name = Some("FootCtrl Mini".into());
    settings.queue_capacity = 10;
    rig.service.save(&settings)?;

    let loaded = ConfigStore::new(rig.directory.join("config.json")).load()?;
    assert_eq!(loaded.settings(), settings);
    assert_eq!(rig.service.view(Vec::new()).settings, settings);
    assert_eq!(loaded.midi.notes.len(), 8);
    let _ = std::fs::remove_dir_all(&rig.directory);
    Ok(())
}

#[test]
fn a_value_out_of_range_is_refused_and_nothing_changes() {
    let rig = rig("invalid");
    let mut settings = rig.service.view(Vec::new()).settings;
    settings.midi_channel = Some(16);
    let refused = rig.service.save(&settings);
    assert!(
        matches!(refused, Err(ConfigError::Invalid(ref text)) if text.contains("midi.channel"))
    );
    assert_eq!(
        rig.service.view(Vec::new()).settings,
        AppConfig::default().settings()
    );
    assert!(!rig.directory.join("config.json").exists());
}

#[test]
fn an_empty_device_name_means_all_devices() -> Result<(), ConfigError> {
    let rig = rig("empty");
    let mut settings = rig.service.view(Vec::new()).settings;
    settings.midi_device_name = Some(String::new());
    rig.service.save(&settings)?;
    assert_eq!(rig.service.view(Vec::new()).settings.midi_device_name, None);
    let _ = std::fs::remove_dir_all(&rig.directory);
    Ok(())
}

#[test]
fn the_queue_limits_apply_without_a_restart() -> Result<(), ConfigError> {
    let rig = rig("live");
    assert_eq!(rig.bus.dispatch(Command::Next), Ok(()));
    rig.clock.advance(Duration::from_millis(100));
    assert_eq!(rig.bus.dispatch(Command::Next), Ok(()));
    assert_eq!(
        rig.driver.sent().len(),
        1,
        "repeat inside the default window"
    );

    let mut settings = rig.service.view(Vec::new()).settings;
    settings.queue_repeat_window_milliseconds = 0;
    rig.service.save(&settings)?;
    assert_eq!(rig.bus.dispatch(Command::Next), Ok(()));
    assert_eq!(rig.driver.sent().len(), 2, "no repeat window any more");
    let _ = std::fs::remove_dir_all(&rig.directory);
    Ok(())
}

#[test]
fn a_refused_write_changes_neither_the_values_nor_the_queue() {
    let repository = Arc::new(crate::fake_config_repository::FakeConfigRepository::default());
    let driver = Arc::new(FakeDriver::default());
    let bus = Arc::new(CommandBus::new(
        driver,
        Arc::new(EventBus::default()),
        Arc::new(FakeClock::default()),
        QueueSettings::default(),
    ));
    let service = SettingsService::new(
        repository.clone(),
        AppConfig::default(),
        bus,
        Arc::new(|_| {}),
    );
    let mut settings = service.view(Vec::new()).settings;
    settings.queue_capacity = 10;
    repository.refuse_writes();
    assert!(matches!(service.save(&settings), Err(ConfigError::Io(_))));
    assert_eq!(
        service.view(Vec::new()).settings,
        AppConfig::default().settings()
    );
    assert!(!repository.exists());
}

#[test]
fn only_a_change_in_the_midi_settings_is_passed_on() -> Result<(), ConfigError> {
    let told = Arc::new(Mutex::new(Vec::new()));
    let listener = Arc::clone(&told);
    let repository = Arc::new(crate::fake_config_repository::FakeConfigRepository::default());
    let bus = Arc::new(CommandBus::new(
        Arc::new(FakeDriver::default()),
        Arc::new(EventBus::default()),
        Arc::new(FakeClock::default()),
        QueueSettings::default(),
    ));
    let service = SettingsService::new(
        repository,
        AppConfig::default(),
        bus,
        Arc::new(move |midi| {
            if let Ok(mut told) = listener.lock() {
                told.push(midi.clone());
            }
        }),
    );
    let mut settings = service.view(Vec::new()).settings;
    settings.queue_capacity = 10;
    service.save(&settings)?;
    assert_eq!(told.lock().map(|told| told.len()).unwrap_or_default(), 0);
    settings.midi_channel = Some(3);
    service.save(&settings)?;
    let told = told.lock().map(|told| told.clone()).unwrap_or_default();
    assert_eq!(told.len(), 1);
    assert_eq!(told.first().and_then(|midi| midi.channel), Some(3));
    Ok(())
}
