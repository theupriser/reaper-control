use std::fs;
use std::io;
use std::path::PathBuf;

/// The `faulted` file the extension leaves in its folder when it turns itself off, holding the
/// reason in one line. The app reads it to tell the user why the extension does not answer.
#[derive(Debug)]
pub struct FaultFile {
    path: PathBuf,
}

impl FaultFile {
    /// A fault file at `path`.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Records why the extension is disabled.
    pub fn report(&self, reason: &str) -> io::Result<()> {
        fs::write(&self.path, reason)
    }

    /// Forgets an earlier fault; a missing file is fine.
    pub fn clear(&self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests;
