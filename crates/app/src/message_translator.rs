//! Turns what the link says into what the app announces.

use link::LinkEvent;
use protocol::message::Outcome;

use crate::app_event::AppEvent;

/// The one place that knows the link's vocabulary; the rest of the app only sees `AppEvent`.
pub struct MessageTranslator;

impl MessageTranslator {
    /// The announcement a link event stands for. `None` for state pushes (the view shows those)
    /// and for a command that was done.
    #[must_use]
    pub fn translate(event: &LinkEvent) -> Option<AppEvent> {
        match event {
            LinkEvent::Connected { extension_version } => Some(AppEvent::LinkConnected {
                extension_version: extension_version.clone(),
            }),
            LinkEvent::Disconnected => Some(AppEvent::LinkLost),
            LinkEvent::Ack {
                id,
                outcome: Outcome::Rejected { reason },
            } => Some(AppEvent::ExtensionRefused {
                id: *id,
                reason: reason.clone(),
            }),
            LinkEvent::Event(record) => Some(AppEvent::PerformanceEvent(record.clone())),
            LinkEvent::EventsLost { oldest_available } => Some(AppEvent::EventsMissed {
                oldest_available: *oldest_available,
            }),
            LinkEvent::Ack { .. } | LinkEvent::Live(_) | LinkEvent::Catalog(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
