//! Turns what the link reports into what the UI shows.

use link::LinkEvent;
use protocol::{LinkStatus, LinkView};

/// What the UI shows after the link reported `event`.
#[must_use]
pub fn apply_event(view: LinkView, event: LinkEvent) -> LinkView {
    match event {
        LinkEvent::Connected { extension_version } => LinkView {
            status: LinkStatus::Connected { extension_version },
            ..view
        },
        LinkEvent::State(state) => LinkView { state, ..view },
        LinkEvent::Disconnected => LinkView::default(),
        LinkEvent::Ack { .. } => view,
    }
}

#[cfg(test)]
mod tests {
    use protocol::message::Outcome;
    use protocol::{AppState, Phase};

    use super::*;

    fn playing() -> AppState {
        AppState {
            phase: Phase::Playing,
            position: 4.0,
            ..AppState::default()
        }
    }

    #[test]
    fn connecting_then_a_state_shows_both() {
        let view = apply_event(
            LinkView::default(),
            LinkEvent::Connected {
                extension_version: "1".into(),
            },
        );
        let view = apply_event(view, LinkEvent::State(playing()));
        assert_eq!(
            view.status,
            LinkStatus::Connected {
                extension_version: "1".into()
            }
        );
        assert_eq!(view.state, playing());
    }

    #[test]
    fn losing_the_link_forgets_the_old_state() {
        let view = LinkView {
            status: LinkStatus::Connected {
                extension_version: "1".into(),
            },
            state: playing(),
        };
        assert_eq!(
            apply_event(view, LinkEvent::Disconnected),
            LinkView::default()
        );
    }

    #[test]
    fn an_ack_changes_nothing() {
        let view = LinkView {
            state: playing(),
            ..LinkView::default()
        };
        let acked = apply_event(
            view.clone(),
            LinkEvent::Ack {
                id: 1,
                outcome: Outcome::Done,
            },
        );
        assert_eq!(acked, view);
    }
}
