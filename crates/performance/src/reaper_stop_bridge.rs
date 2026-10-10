use shared_kernel::Seconds;

/// REAPER stopped by itself at a song's `!1008` marker (SWS does that) before the song's end. The
/// performance keeps time from there, on the clock it is given, so the timeline runs on to the end
/// of the song's window and the hard stop happens there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ReaperStopBridge {
    stopped_at: Seconds,
    position: Seconds,
}

impl ReaperStopBridge {
    /// REAPER stopped at `position` when the clock read `now`.
    pub(crate) fn begin(now: Seconds, position: Seconds) -> Self {
        Self {
            stopped_at: now,
            position,
        }
    }

    /// Where the timeline is by now.
    pub(crate) fn position_at(&self, now: Seconds) -> f64 {
        self.position.get() + (now.get() - self.stopped_at.get()).max(0.0)
    }
}
