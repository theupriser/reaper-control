use crate::{Effect, Event};

/// What one step produced: what to do in REAPER and what to record.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Output {
    /// For the adapter, in order.
    pub effects: Vec<Effect>,
    /// For the journal, in order.
    pub events: Vec<Event>,
}
