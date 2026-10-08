//! Whether the app is connected to the extension, and the moments that changes.

use link::LinkEvent;

use crate::app_event::AppEvent;
use crate::message_translator::MessageTranslator;

/// The app's side of one run of connecting, losing and reconnecting. It announces a change of
/// connection once, however often the link repeats it.
#[derive(Debug, Default)]
pub struct LinkSession {
    connected: bool,
}

impl LinkSession {
    /// Takes in one link event and returns what the app should announce because of it.
    pub fn observe(&mut self, event: &LinkEvent) -> Option<AppEvent> {
        let announcement = MessageTranslator::translate(event)?;
        match announcement {
            AppEvent::LinkConnected { .. } if self.connected => None,
            AppEvent::LinkLost if !self.connected => None,
            AppEvent::LinkConnected { .. } => {
                self.connected = true;
                Some(announcement)
            }
            AppEvent::LinkLost => {
                self.connected = false;
                Some(announcement)
            }
            other => Some(other),
        }
    }

    /// Whether the handshake is done and the link has not been lost since.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests;
