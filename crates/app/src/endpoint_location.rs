//! Finding the extension's endpoint file.

use std::path::PathBuf;

/// Where the extension writes `endpoint.json`: `RC2_DIRECTORY` when set (an isolated REAPER),
/// otherwise the `RC2` folder of REAPER's resource path. `None` when the home folder is unknown.
#[must_use]
pub fn endpoint_file() -> Option<PathBuf> {
    let directory = match std::env::var_os("RC2_DIRECTORY") {
        Some(directory) => PathBuf::from(directory),
        None => resource_path()?.join("RC2"),
    };
    Some(directory.join("endpoint.json"))
}

/// The extension's folder: `endpoint.json`, its log and its journal are in it.
#[must_use]
pub fn extension_directory() -> Option<PathBuf> {
    endpoint_file()?.parent().map(std::path::Path::to_path_buf)
}

/// Where the extension writes `faulted` when it turns itself off: next to `endpoint.json`.
#[must_use]
pub fn fault_file() -> Option<PathBuf> {
    Some(endpoint_file()?.with_file_name("faulted"))
}

#[cfg(target_os = "windows")]
fn resource_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join("REAPER"))
}

#[cfg(not(target_os = "windows"))]
fn resource_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support/REAPER"))
}
