use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Appends lines to the extension's log file. Failing to write is ignored: logging must never
/// be the reason something else breaks.
#[derive(Debug)]
pub struct Log {
    path: PathBuf,
}

impl Log {
    /// A log that appends to `path`.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Appends one line.
    pub fn line(&self, text: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "{text}");
        }
    }
}
