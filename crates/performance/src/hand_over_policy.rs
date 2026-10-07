use shared_kernel::{InvalidValue, Seconds};

use crate::SongWindow;

/// When a song counts as over: how far before its end the hand-over fires
/// and how long a following one is ignored (v1 debounce).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandOverPolicy {
    lead: f64,
    debounce: f64,
}

impl Default for HandOverPolicy {
    /// 15 ms lead (ADR-005), 1 s debounce (v1).
    fn default() -> Self {
        Self {
            lead: 0.015,
            debounce: 1.0,
        }
    }
}

impl HandOverPolicy {
    /// Creates a policy; neither value may be negative.
    pub fn new(lead: Seconds, debounce: Seconds) -> Result<Self, InvalidValue> {
        if lead.get() < 0.0 || debounce.get() < 0.0 {
            return Err(InvalidValue::OutOfRange);
        }
        Ok(Self {
            lead: lead.get(),
            debounce: debounce.get(),
        })
    }

    /// Whether the position is within the lead of the song's end (or past it).
    pub fn is_due(&self, position: Seconds, window: &SongWindow) -> bool {
        position.get() >= window.end().get() - self.lead
    }

    /// Whether a hand-over fired too recently to fire another.
    pub fn is_debounced(&self, now: Seconds, last: Option<Seconds>) -> bool {
        last.is_some_and(|last| now.get() - last.get() < self.debounce)
    }
}
