use std::sync::mpsc::Receiver;
use std::sync::{Mutex, PoisonError};

use protocol::Command;

/// Commands waiting to go out, numbered by [`super::LinkClient::send`].
pub(in crate::client) struct CommandQueue(Mutex<Receiver<(u64, Command)>>);

impl CommandQueue {
    pub(in crate::client) fn new(receiver: Receiver<(u64, Command)>) -> Self {
        Self(Mutex::new(receiver))
    }

    /// Anything typed while the link was down is stale.
    pub(in crate::client) fn discard(&self) {
        while self.next().is_some() {}
    }

    pub(in crate::client) fn next(&self) -> Option<(u64, Command)> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .try_recv()
            .ok()
    }
}
