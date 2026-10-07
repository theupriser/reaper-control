use std::time::{Duration, Instant};

/// Decides when to ping and when the extension counts as gone.
pub(in crate::client) struct Heartbeat {
    ping_after: Duration,
    dead_after: Duration,
    last_heard: Instant,
    last_ping: Instant,
}

impl Heartbeat {
    pub(in crate::client) fn new(ping_after: Duration, dead_after: Duration) -> Self {
        let now = Instant::now();
        Self {
            ping_after,
            dead_after,
            last_heard: now,
            last_ping: now,
        }
    }

    pub(in crate::client) fn heard(&mut self) {
        self.last_heard = Instant::now();
    }

    pub(in crate::client) fn is_dead(&self) -> bool {
        self.last_heard.elapsed() > self.dead_after
    }

    /// True once per quiet period.
    pub(in crate::client) fn ping_due(&mut self) -> bool {
        let due = self.last_heard.elapsed() > self.ping_after
            && self.last_ping.elapsed() > self.ping_after;
        if due {
            self.last_ping = Instant::now();
        }
        due
    }
}
