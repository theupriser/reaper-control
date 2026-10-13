use std::path::Path;
use std::process::{Command, Stdio};

use super::Launcher;

/// Starts the app as its own process: `open` for a macOS application bundle, the program itself
/// on Windows. Nothing here waits for the app, so REAPER's thread is never held.
#[derive(Debug, Default)]
pub struct ProcessLauncher;

impl Launcher for ProcessLauncher {
    fn launch(&mut self, application: &Path) -> Result<(), String> {
        if !application.exists() {
            return Err(format!("{} does not exist", application.display()));
        }
        let mut command = program_for(application);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        // Collect the exit status elsewhere so no zombie is left behind.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn program_for(application: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg(application);
    command
}

#[cfg(target_os = "windows")]
fn program_for(application: &Path) -> Command {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    let mut command = Command::new(application);
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    command
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn program_for(application: &Path) -> Command {
    Command::new(application)
}
