use shared_kernel::{Bpm, Seconds};

use crate::{InvalidTempoMap, TempoSegment, TimeSignature};

/// A snapshot of the project's tempo and time signature changes.
#[derive(Debug, Clone, PartialEq)]
pub struct TempoMap {
    segments: Vec<TempoSegment>,
}

impl TempoMap {
    /// Builds a map; segments must ascend and the first must cover time 0.
    pub fn new(segments: Vec<TempoSegment>) -> Result<Self, InvalidTempoMap> {
        let first = segments.first().ok_or(InvalidTempoMap::Empty)?;
        if first.start().get() > 0.0 {
            return Err(InvalidTempoMap::DoesNotCoverStart);
        }
        let ascending = segments
            .windows(2)
            .all(|pair| matches!(pair, [a, b] if a.start() < b.start()));
        if !ascending {
            return Err(InvalidTempoMap::NotAscending);
        }
        Ok(Self { segments })
    }

    /// One tempo and signature for the whole timeline.
    pub fn constant(bpm: Bpm, signature: TimeSignature) -> Self {
        Self {
            segments: vec![TempoSegment::new(Seconds::ZERO, bpm, signature)],
        }
    }

    /// The signature in force at `time` (the first segment before the start).
    pub fn signature_at(&self, time: Seconds) -> TimeSignature {
        self.segments
            .iter()
            .rev()
            .find(|s| s.start() <= time)
            .or_else(|| self.segments.first())
            .map_or_else(TimeSignature::common, TempoSegment::signature)
    }

    /// How far before `cue` a run of `bars` bars begins, walking back through
    /// every tempo and signature change. Never longer than `cue` itself.
    pub fn bars_before(&self, cue: Seconds, bars: u32) -> Seconds {
        let mut at = cue.get();
        let mut remaining = f64::from(bars);
        for segment in self.segments.iter().rev().filter(|s| s.start() < cue) {
            let bar = segment.bar_seconds();
            let room = (at - segment.start().get()) / bar;
            if remaining <= room {
                at -= remaining * bar;
                remaining = 0.0;
                break;
            }
            remaining -= room;
            at = segment.start().get();
        }
        let span = if remaining > 0.0 {
            cue.get()
        } else {
            cue.get() - at
        };
        Seconds::new(span.clamp(0.0, cue.get().max(0.0))).unwrap_or(Seconds::ZERO)
    }
}
