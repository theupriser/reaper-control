//! One REAPER the extension can be installed into.

use std::path::PathBuf;

use crate::install_platform::InstallPlatform;

/// A REAPER resource folder, and the program that goes with it when it is known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReaperInstall {
    /// Where REAPER keeps its settings and `UserPlugins`.
    pub resource_folder: PathBuf,
    /// The REAPER program, if found.
    pub executable: Option<PathBuf>,
}

impl ReaperInstall {
    /// The REAPER in `chosen` (a folder given with `--reaper-folder`), or the normal install
    /// for the user whose folder is `user_folder`.
    #[must_use]
    pub fn locate(
        platform: InstallPlatform,
        chosen: Option<PathBuf>,
        user_folder: &std::path::Path,
    ) -> Self {
        match chosen {
            Some(folder) => {
                let executable = platform.executable_in(&folder);
                Self {
                    executable: executable.exists().then_some(executable),
                    resource_folder: folder,
                }
            }
            None => {
                let executable = platform.default_executable();
                Self {
                    resource_folder: platform.default_resource_folder(user_folder),
                    executable: executable.exists().then_some(executable),
                }
            }
        }
    }
}
