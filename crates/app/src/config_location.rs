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

/// The folder of the restore-only setlist copies, `setlist-mirror` next to the config file.
#[must_use]
pub fn mirror_directory() -> Option<PathBuf> {
    Some(app_directory()?.join("setlist-mirror"))
}

/// The `setlists` folder of v1 (`RC2_LEGACY_DIRECTORY` replaces v1's data folder, for tests).
#[must_use]
pub fn legacy_setlists_directory() -> Option<PathBuf> {
    let directory = match std::env::var_os("RC2_LEGACY_DIRECTORY") {
        Some(directory) => PathBuf::from(directory),
        None => data_directory()?.join("electron-reaper-control"),
    };
    Some(directory.join("setlists"))
}

#[cfg(target_os = "windows")]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?))
}

#[cfg(not(target_os = "windows"))]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support"))
}
