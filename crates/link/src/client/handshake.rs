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
    /// The extension's version when it accepted us, `None` when it did not.
    pub(super) fn perform(&self, link: &mut Link, token: String) -> Option<String> {
        link.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            token,
            resume_from_event_id: None,
        })?;
        let started = Instant::now();
        while started.elapsed() < WELCOME_TIMEOUT && !self.stop.load(Ordering::SeqCst) {
            match link.poll().ok()? {
                Some(ServerMessage::Welcome {
                    protocol,
                    extension_version,
                    ..
                }) if protocol == PROTOCOL_VERSION => return Some(extension_version),
                Some(_) => return None,
                None => {}
            }
        }
        None
    }
}
