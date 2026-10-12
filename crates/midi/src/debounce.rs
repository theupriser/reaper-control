//! Ignoring a note that was pressed a moment ago.

use std::collections::HashMap;
use std::time::Duration;

/// Remembers when each note last counted, across all devices, so one press heard twice acts once.
#[derive(Debug)]
pub struct MidiDebounce {
    window: Duration,
    last: HashMap<u8, Duration>,
}

impl MidiDebounce {
    /// A debounce that ignores a note repeated within `window`.
    #[must_use]
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            last: HashMap::new(),
        }
    }

    /// Whether this press counts; it does when the note was not counted within the window before `now`.
    pub fn allows(&mut self, note: u8, now: Duration) -> bool {
        if let Some(before) = self.last.get(&note)
            && now.saturating_sub(*before) < self.window
        {
            return false;
        }
        self.last.insert(note, now);
        true
    }
}
