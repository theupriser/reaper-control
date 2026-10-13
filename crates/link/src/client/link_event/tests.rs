use protocol::message::ServerMessage;

use super::LinkEvent;

#[test]
fn the_closing_message_becomes_the_closing_event() {
    assert_eq!(
        LinkEvent::from_server(ServerMessage::Closing),
        Some(LinkEvent::Closing)
    );
}
