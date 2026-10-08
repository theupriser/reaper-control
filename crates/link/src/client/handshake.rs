use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use protocol::message::{ClientMessage, PROTOCOL_VERSION, ServerMessage};

use super::link::Link;

const WELCOME_TIMEOUT: Duration = Duration::from_secs(2);

/// Sends the Hello and waits for the Welcome.
pub(super) struct Handshake<'a> {
    pub(super) stop: &'a AtomicBool,
}

impl Handshake<'_> {
    /// The extension's version and newest event id when it accepted us, `None` when it did not.
    pub(super) fn perform(
        &self,
        link: &mut Link,
        token: String,
        resume_from_event_id: Option<u64>,
    ) -> Option<(String, u64)> {
        link.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            token,
            resume_from_event_id,
        })?;
        let started = Instant::now();
        while started.elapsed() < WELCOME_TIMEOUT && !self.stop.load(Ordering::SeqCst) {
            match link.poll().ok()? {
                Some(ServerMessage::Welcome {
                    protocol,
                    extension_version,
                    last_event_id,
                    ..
                }) if protocol == PROTOCOL_VERSION => {
                    return Some((extension_version, last_event_id));
                }
                Some(_) => return None,
                None => {}
            }
        }
        None
    }
}
