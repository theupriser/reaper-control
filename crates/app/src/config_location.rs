//! Finding the app's config file.

use std::path::PathBuf;

/// `config.json` in `RC2_CONFIG_DIRECTORY` when set (tests, an isolated setup), otherwise in the
/// app's own folder in the user's application data. `None` when the home folder is unknown.
#[must_use]
pub fn config_file() -> Option<PathBuf> {
    let directory = match std::env::var_os("RC2_CONFIG_DIRECTORY") {
        Some(directory) => PathBuf::from(directory),
        None => data_directory()?.join("ReaperControl2"),
    };
    Some(directory.join("config.json"))
}

#[cfg(target_os = "windows")]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?))
}

#[cfg(not(target_os = "windows"))]
fn data_directory() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support"))
}
