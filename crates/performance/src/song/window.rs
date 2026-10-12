use shared_kernel::{InvalidValue, Seconds};

/// The stretch of the timeline a song plays: its start and its effective end
/// (the region end, or the start plus `!length`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SongWindow {
    start: Seconds,
    end: Seconds,
}

impl SongWindow {
    /// Creates a window; the end must be after the start.
    pub fn new(start: Seconds, end: Seconds) -> Result<Self, InvalidValue> {
        if end.get() > start.get() {
            Ok(Self { start, end })
        } else {
            Err(InvalidValue::OutOfRange)
        }
    }

    /// Where the song starts.
    pub fn start(&self) -> Seconds {
        self.start
    }

    /// Where the song ends.
    pub fn end(&self) -> Seconds {
        self.end
    }

    /// Whether a position lies in the window, edges included.
    pub fn contains(&self, position: Seconds) -> bool {
        self.start.get() <= position.get() && position.get() <= self.end.get()
    }
}
