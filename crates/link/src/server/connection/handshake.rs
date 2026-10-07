use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use protocol::message::{ClientMessage, check_hello};

use crate::wire::{MessageReader, ReadError};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

/// Waits for the first message and accepts the peer only for a valid Hello.
pub(super) struct Handshake<'a> {
    pub(super) token: &'a str,
    pub(super) stop: &'a AtomicBool,
}

impl Handshake<'_> {
    /// True when the peer may stay; false (close without a reply) when it was too slow or wrong.
    pub(super) fn accepts(&self, reader: &mut MessageReader) -> Result<bool, ReadError> {
        let started = Instant::now();
        let first = loop {
            if self.stop.load(Ordering::SeqCst) || started.elapsed() > HANDSHAKE_TIMEOUT {
                return Ok(false);
            }
            if let Some(message) = reader.poll::<ClientMessage>()? {
                break message;
            }
        };
        Ok(check_hello(self.token, &first).is_ok())
    }
}
