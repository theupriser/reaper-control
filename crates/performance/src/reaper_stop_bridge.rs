use shared_kernel::Seconds;

/// REAPER stopped by itself at a song's `!1008` marker (SWS does that) before the song's end. The
/// performance keeps time from there, on the clock it is given, so the timeline runs on to the end
/// of the song's window and the hard stop happens there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ReaperStopBridge {
    stopped_at: Seconds,
    position: Seconds,
    seen_at: Seconds,
    frozen: bool,
}

impl ReaperStopBridge {
    /// REAPER stopped at `position` when the clock read `now`.
    pub(crate) fn begin(now: Seconds, position: Seconds) -> Self {
        Self {
            stopped_at: now,
            position,
            seen_at: now,
            frozen: false,
        }
    }

    /// The clock read `now` on the latest tick; a pause that comes between ticks freezes there.
    pub(crate) fn see(&mut self, now: Seconds) {
        self.seen_at = now;
    }

    /// The timeline stands still where it was at the latest tick, until `resume`.
    pub(crate) fn freeze(&mut self) {
        if let Ok(position) = Seconds::new(self.position_at(self.seen_at)) {
            self.position = position;
        }
        self.frozen = true;
    }

    /// The timeline runs on again from where it stands.
    pub(crate) fn resume(&mut self, now: Seconds) {
        self.stopped_at = now;
        self.frozen = false;
    }

    /// Whether the timeline stands still.
    pub(crate) fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Where the timeline is by now.
    pub(crate) fn position_at(&self, now: Seconds) -> f64 {
        if self.frozen {
            return self.position.get();
        }
        self.position.get() + (now.get() - self.stopped_at.get()).max(0.0)
    }
}
