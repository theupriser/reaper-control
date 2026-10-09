//! The config file: defaults, migration, validation, atomic save.

use app::AppConfig;
use app::ConfigError;
use app::ConfigRepository;
use app::ConfigStore;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn store(name: &str) -> Result<(ConfigStore, std::path::PathBuf), std::io::Error> {
    let directory = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    Ok((ConfigStore::new(directory.join("config.json")), directory))
}

#[test]
fn no_file_gives_the_defaults() -> TestResult {
    let (store, directory) = store("config-none")?;
    assert_eq!(store.load()?, AppConfig::default());
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn a_saved_config_comes_back_and_leaves_no_temporary_file() -> TestResult {
    let (store, directory) = store("config-save")?;
    let mut config = AppConfig::default();
    config.queue.capacity = 8;
    config.queue.timeout_milliseconds = 2000;
    store.save(&config)?;
    assert_eq!(store.load()?, config);
    assert_eq!(config.queue.settings().capacity, 8);
    assert_eq!(std::fs::read_dir(&directory)?.count(), 1);
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn a_file_without_a_version_is_migrated_and_missing_fields_take_defaults() -> TestResult {
    let (store, directory) = store("config-old")?;
    std::fs::write(directory.join("config.json"), r#"{"queue":{"capacity":4}}"#)?;
    let config = store.load()?;
    assert_eq!(config.schema_version, 1);
    assert_eq!(config.queue.capacity, 4);
    assert_eq!(config.queue.timeout_milliseconds, 5000);
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn bad_files_are_refused_with_a_reason() -> TestResult {
    let (store, directory) = store("config-bad")?;
    let file = directory.join("config.json");
    std::fs::write(&file, r#"{"queue":{"capacity":0}}"#)?;
    assert!(matches!(store.load(), Err(ConfigError::Invalid(_))));
    std::fs::write(&file, r#"{"schema_version":99}"#)?;
    assert!(matches!(
        store.load(),
        Err(ConfigError::Newer { found: 99, .. })
    ));
    std::fs::write(&file, "not json")?;
    assert!(matches!(store.load(), Err(ConfigError::Parse(_))));
    std::fs::write(&file, "[1]")?;
    assert!(matches!(store.load(), Err(ConfigError::Invalid(_))));
    let mut config = AppConfig::default();
    config.queue.repeat_window_milliseconds = 9000;
    assert!(matches!(store.save(&config), Err(ConfigError::Invalid(_))));
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn the_first_start_takes_the_v1_midi_settings_once() -> TestResult {
    use app::import_legacy_config;
    let directory = std::env::temp_dir().join(format!("app-config-import-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    let legacy = directory.join("v1.json");
    std::fs::write(
        &legacy,
        r#"{"midi":{"enabled":true,"deviceName":"FootCtrl Mini","channel":3,"noteMapping":{"60":"playPause","61":"nonsense"}},"host":"127.0.0.1"}"#,
    )?;
    let store = ConfigStore::new(directory.join("config.json"));

    assert!(import_legacy_config(&store, &directory.join("missing.json")).is_none());
    assert!(!store.exists());

    let imported = import_legacy_config(&store, &legacy).ok_or("nothing imported")?;
    assert_eq!(imported.midi.device_name.as_deref(), Some("FootCtrl Mini"));
    assert_eq!(imported.midi.channel, Some(3));
    assert_eq!(store.load()?, imported);

    assert!(import_legacy_config(&store, &legacy).is_none());
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn the_log_level_is_saved_and_an_unknown_one_is_refused() -> TestResult {
    let (store, directory) = store("config-log-level")?;
    assert_eq!(AppConfig::default().log.level, "info");
    let mut config = AppConfig::default();
    config.log.level = "debug".into();
    store.save(&config)?;
    assert_eq!(store.load()?.log.level, "debug");
    assert_eq!(
        AppConfig::default()
            .with_settings(&config.settings())
            .log
            .level,
        "debug"
    );

    config.log.level = "chatty".into();
    assert!(matches!(store.save(&config), Err(ConfigError::Invalid(_))));
    std::fs::remove_dir_all(directory)?;
    Ok(())
}
