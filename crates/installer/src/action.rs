//! One step the installer would take.

use std::path::PathBuf;

/// One step the installer would take.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstallAction {
    /// Keep the file that is installed now, so it can be restored.
    KeepPrevious {
        /// The installed file.
        from: PathBuf,
        /// Where the copy is kept.
        to: PathBuf,
    },
    /// Put the extension that ships with the app into `UserPlugins`.
    CopyExtension {
        /// The bundled file.
        from: PathBuf,
        /// Where REAPER will load it from.
        to: PathBuf,
    },
    /// Sign it and clear quarantine (macOS).
    PrepareLibrary {
        /// The installed file.
        library: PathBuf,
    },
}
