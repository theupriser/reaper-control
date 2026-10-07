use shared_kernel::{Bpm, Seconds};

use crate::TempoMap;

/// How long a count-in lasts, in bars. v1 used two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountIn {
    bars: u32,
}

impl CountIn {
    /// A count-in of `bars` bars.
    pub fn new(bars: u32) -> Self {
        Self { bars }
    }

    /// The lead-in for `Input::SeekCue`: how long before `cue` the cursor
    /// starts. A `!bpm` tempo wins over the project's tempo map.
    pub fn lead_in(&self, map: &TempoMap, cue: Seconds, tempo: Option<Bpm>) -> Seconds {
        match tempo {
            Some(bpm) => map.bars_before_at(cue, self.bars, bpm),
            None => map.bars_before(cue, self.bars),
        }
    }
}

impl Default for CountIn {
    fn default() -> Self {
        Self::new(2)
    }
}
