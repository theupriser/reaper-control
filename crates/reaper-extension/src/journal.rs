use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use protocol::WireEvent;

use crate::log::Log;

/// A journal that grows past this many bytes is set aside as `journal.previous.log` at the next
/// start, so it never fills the disk.
const LARGEST_BYTES: u64 = 1_000_000;

/// The hand-over journal (SPEC S-8): one line per event, written by the extension because the app
/// may be gone during a show. The app reads it afterwards.
#[derive(Debug)]
pub struct Journal {
    log: Log,
}

impl Journal {
    /// A journal that appends to `path`; an oversized earlier journal is set aside first.
    pub fn new(path: PathBuf) -> Self {
        if std::fs::metadata(&path).is_ok_and(|metadata| metadata.len() > LARGEST_BYTES) {
            let _ = std::fs::rename(&path, path.with_file_name("journal.previous.log"));
        }
        Self {
            log: Log::new(path),
        }
    }

    /// Appends one line: the wall clock in seconds and the event, as JSON.
    pub fn record(&self, event: &WireEvent) {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |elapsed| elapsed.as_secs_f64());
        if let Ok(event) = serde_json::to_string(event) {
            self.log
                .line(&format!("{{\"timestamp\":{seconds:.3},\"event\":{event}}}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn events_become_lines_and_a_large_journal_is_set_aside() -> TestResult {
        let directory = std::env::temp_dir().join(format!("journal-{}", std::process::id()));
        std::fs::create_dir_all(&directory)?;
        let path = directory.join("journal.log");

        let journal = Journal::new(path.clone());
        journal.record(&WireEvent::PerformanceStarted);
        journal.record(&WireEvent::HandOverStarted {
            from: "a".into(),
            to: "b".into(),
        });
        let text = std::fs::read_to_string(&path)?;
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "got {text:?}");
        assert!(lines[0].contains("\"PerformanceStarted\""), "got {text:?}");
        assert!(lines[1].contains("\"to\":\"b\""), "got {text:?}");

        std::fs::write(&path, vec![b'x'; 1_000_001])?;
        let _fresh = Journal::new(path.clone());
        assert!(!path.exists());
        assert!(directory.join("journal.previous.log").exists());
        std::fs::remove_dir_all(&directory)?;
        Ok(())
    }
}
