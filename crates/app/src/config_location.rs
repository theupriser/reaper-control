//! Finding the app's own files and the folder v1 left behind.

use std::path::PathBuf;

fn app_directory() -> Option<PathBuf> {
    match std::env::var_os("RC2_CONFIG_DIRECTORY") {
        Some(directory) => Some(PathBuf::from(directory)),
        None => Some(data_directory()?.join("ReaperControl2")),
    }
}

/// `config.json` in `RC2_CONFIG_DIRECTORY` when set (tests, an isolated setup), otherwise in the
/// app's own folder in the user's application data. `None` when the home folder is unknown.
#[must_use]
pub fn config_file() -> Option<PathBuf> {
    Some(app_directory()?.join("config.json"))
}

/// The folder of the log files, `logs` next to the config file.
#[must_use]
pub fn log_directory() -> Option<PathBuf> {
    Some(app_directory()?.join("logs"))
}

/// The folder the diagnostics bundles are written to, `diagnostics` next to the config file.
#[must_use]
pub fn diagnostics_directory() -> Option<PathBuf> {
    Some(app_directory()?.join("diagnostics"))
}

/// The folder of the restore-only setlist copies, `setlist-mirror` next to the config file.
#[must_use]
pub fn mirror_directory() -> Option<PathBuf> {
    Some(app_directory()?.join("setlist-mirror"))
}

/// The `setlists` folder of v1.
#[must_use]
pub fn legacy_setlists_directory() -> Option<PathBuf> {
    Some(legacy_directory()?.join("setlists"))
}

/// The `config.json` of v1.
#[must_use]
pub fn legacy_config_file() -> Option<PathBuf> {
    Some(legacy_directory()?.join("config.json"))
}

/// v1's data folder; `RC2_LEGACY_DIRECTORY` replaces it, for tests. The installed v1 app keeps
/// its data in `reaper-control`; `electron-reaper-control` is what a development run of v1 uses.
fn legacy_directory() -> Option<PathBuf> {
    if let Some(directory) = std::env::var_os("RC2_LEGACY_DIRECTORY") {
        return Some(PathBuf::from(directory));
    }
    let data = data_directory()?;
    let installed = data.join("reaper-control");
    if installed.is_dir() {
        return Some(installed);
    }
    Some(data.join("electron-reaper-control"))
}

#[cfg(target_os = "windows")]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?))
}

#[cfg(not(target_os = "windows"))]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support"))
}
