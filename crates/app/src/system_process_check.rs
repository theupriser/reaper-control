//! The real process check.

use std::process::Command;

use crate::process_check::ProcessCheck;

/// Asks the operating system with `pgrep` (macOS) or `tasklist` (Windows). When the question
/// cannot be asked it answers "not running".
pub struct SystemProcessCheck;

impl ProcessCheck for SystemProcessCheck {
    fn reaper_is_running(&self) -> bool {
        #[cfg(windows)]
        let output = {
            use std::os::windows::process::CommandExt;
            // CREATE_NO_WINDOW: no console window flashes up
            Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq reaper.exe", "/NH"])
                .creation_flags(0x0800_0000)
                .output()
                .map(|output| {
                    String::from_utf8_lossy(&output.stdout)
                        .to_lowercase()
                        .contains("reaper.exe")
                })
        };
        #[cfg(not(windows))]
        let output = Command::new("pgrep")
            .args(["-x", "REAPER"])
            .output()
            .map(|output| output.status.success());
        output.unwrap_or(false)
    }
}
