mod replay;

use std::collections::VecDeque;

use protocol::{EventRecord, WireEvent};

pub use replay::Replay;

/// The newest events, numbered, so an app that was away can ask for what it missed.
/// The extension keeps one; older events are in the journal (SPEC §2.2).
#[derive(Debug, Clone)]
pub struct EventLog {
    capacity: usize,
    last_id: u64,
    ring: VecDeque<EventRecord>,
}

impl EventLog {
    /// A log that keeps the newest `capacity` events (at least one).
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            last_id: 0,
            ring: VecDeque::new(),
        }
    }

    /// Numbers the event, keeps it and returns it for sending.
    pub fn push(&mut self, event: WireEvent) -> EventRecord {
        self.last_id += 1;
        let record = EventRecord {
            id: self.last_id,
            event,
        };
        if self.ring.len() == self.capacity {
            self.ring.pop_front();
        }
        self.ring.push_back(record.clone());
        record
    }

    /// Id of the newest event, 0 when there is none yet.
    pub fn last_id(&self) -> u64 {
        self.last_id
    }

    /// What came after the event `after` (0 when the app saw none).
    pub fn since(&self, after: u64) -> Replay {
        let oldest = self.ring.front().map_or(self.last_id + 1, |first| first.id);
        if after > self.last_id || after + 1 < oldest {
            return Replay::Lost {
                oldest_available: oldest,
            };
        }
        Replay::Events(
            self.ring
                .iter()
                .filter(|record| record.id > after)
                .cloned()
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests;
