//! The real REAPER control.

use std::process::Command;

use crate::ReaperControl;
use crate::ReaperInstall;

/// macOS: AppleScript `quit` and `open`. Windows: `taskkill` without `/F` (a close request, so REAPER
/// can ask to save) and starting `reaper.exe`. Both name the program, not one process, so for a REAPER
/// in a chosen folder (`--reaper-folder`) the app does not quit it: it could be another REAPER.
pub struct SystemReaperControl {
    chosen_folder: bool,
}

impl SystemReaperControl {
    /// A control for the normal REAPER, or for the one in a chosen folder.
    #[must_use]
    pub fn new(chosen_folder: bool) -> Self {
        Self { chosen_folder }
    }
}

impl ReaperControl for SystemReaperControl {
    fn quit(&self) -> Result<(), String> {
        if self.chosen_folder {
            return Err("this REAPER was chosen by folder; close it yourself".to_owned());
        }
        #[cfg(windows)]
        let result = {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW: no console window flashes up
            Command::new("taskkill")
                .args(["/IM", "reaper.exe"])
                .creation_flags(0x0800_0000)
                .output()
        };
        #[cfg(not(windows))]
        let result = Command::new("osascript")
            .args(["-e", "tell application id \"com.cockos.reaper\" to quit"])
            .output();
        match result {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
            Err(error) => Err(error.to_string()),
        }
    }

    fn launch(&self, install: &ReaperInstall) -> Result<(), String> {
        let executable = install
            .executable
            .as_ref()
            .ok_or_else(|| "REAPER's program was not found".to_owned())?;
        let settings = self
            .chosen_folder
            .then(|| install.resource_folder.join("reaper.ini"));
        #[cfg(windows)]
        let mut command = Command::new(executable);
        #[cfg(not(windows))]
        let mut command = {
            let mut command = Command::new("open");
            command.arg("-a").arg(application_bundle(executable));
            if settings.is_some() {
                command.arg("-n").arg("--args");
            }
            command
        };
        if let Some(settings) = settings {
            command.arg("-cfgfile").arg(settings);
        }
        let result = command.spawn();
        result.map(|_| ()).map_err(|error| error.to_string())
    }
}

/// `…/REAPER.app/Contents/MacOS/REAPER` becomes `…/REAPER.app`; anything else is passed on as it is.
#[cfg(not(windows))]
fn application_bundle(executable: &std::path::Path) -> std::path::PathBuf {
    executable
        .ancestors()
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .unwrap_or(executable)
        .to_path_buf()
}
