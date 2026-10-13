use std::io::{self, ErrorKind};
use std::path::Path;

/// Turns the extension back on after it switched itself off: removes the `faulted` file it left
/// for the app and the `running` file it found after a crash. The next start of REAPER then loads
/// it normally. Files that are not there are fine.
pub fn reset_fault(directory: &Path) -> io::Result<()> {
    for name in ["faulted", "running"] {
        match std::fs::remove_file(directory.join(name)) {
            Err(error) if error.kind() != ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
