use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use protocol::message::{ClientMessage, check_hello};

use super::admitted::Admitted;
use crate::wire::{MessageReader, ReadError};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);

/// Waits for the first message and accepts the peer only for a valid Hello.
pub(super) struct Handshake<'a> {
    pub(super) token: &'a str,
    pub(super) stop: &'a AtomicBool,
}

impl Handshake<'_> {
    /// Who the peer is when it may stay; `None` (close without a reply) when it was too slow or wrong.
    pub(super) fn admit(&self, reader: &mut MessageReader) -> Result<Option<Admitted>, ReadError> {
        let started = Instant::now();
        let first = loop {
            if self.stop.load(Ordering::SeqCst) || started.elapsed() > HANDSHAKE_TIMEOUT {
                return Ok(None);
            }
            if let Some(message) = reader.poll::<ClientMessage>()? {
                break message;
            }
        };
        if check_hello(self.token, &first).is_err() {
            return Ok(None);
        }
        Ok(match first {
            ClientMessage::Hello {
                resume_from_event_id,
                ..
            } => Some(Admitted {
                resume_from: resume_from_event_id,
            }),
            _ => None,
        })
    }
}
