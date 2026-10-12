//! Copies the extension's journal into the app's log, so one log tells the whole story.

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Mutex;

/// Reads `journal.log` and logs every line it has not logged before.
pub struct JournalImport {
    file: PathBuf,
    offset: Mutex<u64>,
}

impl JournalImport {
    /// An import of this journal that starts at its beginning.
    #[must_use]
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            offset: Mutex::new(0),
        }
    }

    /// Logs the lines written since the last call and returns how many that was. A last line that
    /// is still being written waits for the next call; a journal that was replaced by a shorter
    /// one is read again from its start; no journal means nothing to import.
    pub fn import(&self) -> usize {
        let mut offset = self
            .offset
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Ok(mut file) = std::fs::File::open(&self.file) else {
            return 0;
        };
        let length = file.metadata().map_or(0, |metadata| metadata.len());
        if length < *offset {
            *offset = 0;
        }
        let mut fresh = Vec::new();
        if file.seek(SeekFrom::Start(*offset)).is_err() || file.read_to_end(&mut fresh).is_err() {
            return 0;
        }
        let complete = fresh
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |last| last + 1);
        let text = String::from_utf8_lossy(fresh.get(..complete).unwrap_or_default());
        let mut count = 0;
        for line in text.lines().filter(|line| !line.is_empty()) {
            tracing::info!(target: "journal", "{line}");
            count += 1;
        }
        *offset += complete as u64;
        count
    }
}
