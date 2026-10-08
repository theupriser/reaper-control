use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::start_up::StartUp;

/// The `running` file written at start and removed at clean shutdown. Finding it at the next
/// start means REAPER crashed or was killed (SPEC S-9.4).
#[derive(Debug)]
pub struct SafeModeMarker {
    path: PathBuf,
}

impl SafeModeMarker {
    /// Decides how to start. Creates the marker with `create_new`, so two REAPER instances
    /// cannot both claim it. An error means the marker could not be written, so a crash would
    /// go unnoticed; the caller must not start.
    pub fn claim(path: &Path) -> io::Result<StartUp> {
        let created = OpenOptions::new().write(true).create_new(true).open(path);
        match created {
            Ok(mut file) => {
                file.write_all(std::process::id().to_string().as_bytes())?;
                Ok(StartUp::Normal(Self {
                    path: path.to_path_buf(),
                }))
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(StartUp::SafeMode),
            Err(e) => Err(e),
        }
    }

    /// Removes the marker. Called from the process exit hook, so it must not panic.
    pub fn release(&self) {
        let _ = fs::remove_file(&self.path);
    }

    /// Where the marker lives.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests;
