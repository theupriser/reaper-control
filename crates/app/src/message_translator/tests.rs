use protocol::message::Outcome;
use protocol::{Catalog, EventRecord, Live, WireEvent};

use super::*;

#[test]
fn connecting_and_losing_the_link_are_announced() {
    let connected = LinkEvent::Connected {
        extension_version: "1".into(),
    };
    assert_eq!(
        MessageTranslator::translate(&connected),
        Some(AppEvent::LinkConnected {
            extension_version: "1".into()
        })
    );
    assert_eq!(
        MessageTranslator::translate(&LinkEvent::Disconnected),
        Some(AppEvent::LinkLost)
    );
}

#[test]
fn a_refused_command_is_announced_with_its_id_and_reason() {
    let refused = LinkEvent::Ack {
        id: 7,
        outcome: Outcome::Rejected {
            reason: "this is the last song".into(),
        },
    };
    assert_eq!(
        MessageTranslator::translate(&refused),
        Some(AppEvent::ExtensionRefused {
            id: 7,
            reason: "this is the last song".into()
        })
    );
}

#[test]
fn a_command_that_was_done_is_not_announced() {
    let done = LinkEvent::Ack {
        id: 7,
        outcome: Outcome::Done,
    };
    assert_eq!(MessageTranslator::translate(&done), None);
}

#[test]
fn performance_events_and_gaps_are_announced() {
    let record = EventRecord {
        id: 3,
        event: WireEvent::PerformanceStarted,
    };
    assert_eq!(
        MessageTranslator::translate(&LinkEvent::Event(record.clone())),
        Some(AppEvent::PerformanceEvent(record))
    );
    assert_eq!(
        MessageTranslator::translate(&LinkEvent::EventsLost {
            oldest_available: 40
        }),
        Some(AppEvent::EventsMissed {
            oldest_available: 40
        })
    );
}

#[test]
fn state_pushes_are_left_to_the_view() {
    assert_eq!(
        MessageTranslator::translate(&LinkEvent::Live(Live::default())),
        None
    );
    assert_eq!(
        MessageTranslator::translate(&LinkEvent::Catalog(Catalog::default())),
        None
    );
}
