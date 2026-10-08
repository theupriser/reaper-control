use protocol::message::Outcome;

use super::*;

fn connected() -> LinkEvent {
    LinkEvent::Connected {
        extension_version: "1".into(),
    }
}

#[test]
fn a_new_session_is_not_connected_and_loss_before_connecting_is_not_news() {
    let mut session = LinkSession::default();
    assert!(!session.is_connected());
    assert_eq!(session.observe(&LinkEvent::Disconnected), None);
}

#[test]
fn connecting_then_losing_announces_each_once() {
    let mut session = LinkSession::default();
    assert_eq!(
        session.observe(&connected()),
        Some(AppEvent::LinkConnected {
            extension_version: "1".into()
        })
    );
    assert!(session.is_connected());
    assert_eq!(session.observe(&connected()), None);
    assert_eq!(
        session.observe(&LinkEvent::Disconnected),
        Some(AppEvent::LinkLost)
    );
    assert!(!session.is_connected());
    assert_eq!(session.observe(&LinkEvent::Disconnected), None);
}

#[test]
fn reconnecting_is_announced_again() {
    let mut session = LinkSession::default();
    session.observe(&connected());
    session.observe(&LinkEvent::Disconnected);
    assert!(session.observe(&connected()).is_some());
}

#[test]
fn other_events_pass_through_in_any_state() {
    let mut session = LinkSession::default();
    let refused = LinkEvent::Ack {
        id: 2,
        outcome: Outcome::Rejected {
            reason: "no".into(),
        },
    };
    assert_eq!(
        session.observe(&refused),
        Some(AppEvent::ExtensionRefused {
            id: 2,
            reason: "no".into()
        })
    );
}
