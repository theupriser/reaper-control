use shared_kernel::Seconds;

use crate::{Applied, Freshness};

/// The heartbeat is at least 1 Hz (SPEC §2.2); a second of silence is stale.
const STALE_AFTER: f64 = 1.0;

/// Tracks the `Live` updates the app has shown: drops duplicates and updates
/// that arrive out of order, and says when the picture has gone stale.
#[derive(Debug, Clone, Copy, Default)]
pub struct LiveFeed {
    last: Option<(u64, Seconds)>,
}

impl LiveFeed {
    /// A feed that has seen nothing, so it is stale.
    pub fn new() -> Self {
        Self::default()
    }

    /// Offers update number `sequence`, received at `now`.
    pub fn accept(&mut self, sequence: u64, now: Seconds) -> Applied {
        match self.last {
            Some((last, _)) if sequence <= last => Applied::Ignored,
            _ => {
                self.last = Some((sequence, now));
                Applied::Accepted
            }
        }
    }

    /// Forgets everything: after a reconnect the extension may count again.
    pub fn reset(&mut self) {
        self.last = None;
    }

    /// The sequence number of the update on screen.
    pub fn sequence(&self) -> Option<u64> {
        self.last.map(|(sequence, _)| sequence)
    }

    /// Fresh while the last update is at most a second old.
    pub fn freshness(&self, now: Seconds) -> Freshness {
        match self.last {
            Some((_, at)) if now.get() - at.get() <= STALE_AFTER => Freshness::Fresh,
            _ => Freshness::Stale,
        }
    }
}
