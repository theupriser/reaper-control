//! Finding the extension's endpoint file.

use std::path::PathBuf;

use crate::reaper_folder_argument::reaper_folder_argument;

/// Where the extension writes `endpoint.json`: `RC2_DIRECTORY` when set (an isolated REAPER),
/// else the `RC2` folder of the REAPER named by `--reaper-folder <path>` (one shortcut per
/// REAPER, ADR-006), else the `RC2` folder of REAPER's normal resource path. `None` when the home
/// folder is unknown.
#[must_use]
pub fn endpoint_file() -> Option<PathBuf> {
    endpoint_file_from(
        std::env::var_os("RC2_DIRECTORY").map(PathBuf::from),
        reaper_folder_argument(std::env::args().skip(1)),
        resource_path(),
    )
}

/// The rule behind [`endpoint_file`], with its three sources given: the first that is present
/// wins.
#[must_use]
pub fn endpoint_file_from(
    directory: Option<PathBuf>,
    chosen_reaper_folder: Option<PathBuf>,
    resource_path: Option<PathBuf>,
) -> Option<PathBuf> {
    let directory = directory
        .or_else(|| chosen_reaper_folder.map(|folder| folder.join("RC2")))
        .or_else(|| resource_path.map(|path| path.join("RC2")))?;
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
