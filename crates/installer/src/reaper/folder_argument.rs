//! The `--reaper-folder` start-up argument: one shortcut per REAPER (ADR-006).

use std::path::PathBuf;

const FLAG: &str = "--reaper-folder";

/// The folder named by `--reaper-folder <path>` or `--reaper-folder=<path>`, if any.
#[must_use]
pub fn reaper_folder_argument(arguments: impl IntoIterator<Item = String>) -> Option<PathBuf> {
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        if argument == FLAG {
            return arguments
                .next()
                .filter(|path| !path.is_empty())
                .map(PathBuf::from);
        }
        if let Some(path) = argument.strip_prefix("--reaper-folder=") {
            return (!path.is_empty()).then(|| PathBuf::from(path));
        }
    }
    None
}
