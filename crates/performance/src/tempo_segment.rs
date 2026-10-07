use shared_kernel::{Bpm, Seconds};

use crate::TimeSignature;

/// A stretch of the timeline with one tempo and one time signature.
/// The tempo counts quarter notes per minute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TempoSegment {
    start: Seconds,
    bpm: Bpm,
    signature: TimeSignature,
}

impl TempoSegment {
    /// Creates a segment that begins at `start`.
    pub fn new(start: Seconds, bpm: Bpm, signature: TimeSignature) -> Self {
        Self {
            start,
            bpm,
            signature,
        }
    }

    /// Where the segment begins.
    pub fn start(&self) -> Seconds {
        self.start
    }

    /// The segment's time signature.
    pub fn signature(&self) -> TimeSignature {
        self.signature
    }

    /// Seconds in one bar of this segment.
    pub fn bar_seconds(&self) -> f64 {
        bar_seconds(self.signature, self.bpm)
    }
}

/// Seconds in one bar of `signature` at `bpm`.
pub(crate) fn bar_seconds(signature: TimeSignature, bpm: Bpm) -> f64 {
    signature.quarters_per_bar() * 60.0 / bpm.get()
}
