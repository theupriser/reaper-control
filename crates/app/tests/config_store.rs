//! The config file: defaults, migration, validation, atomic save.

use app::app_config::AppConfig;
use app::config_error::ConfigError;
use app::config_store::ConfigStore;

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
