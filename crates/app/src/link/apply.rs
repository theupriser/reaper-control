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
        // A state that arrives after a newer one (frames can be reordered) is stale.
        LinkEvent::Live(live)
            if view
                .live
                .as_ref()
                .is_some_and(|shown| shown.sequence >= live.sequence) =>
        {
            view
        }
        LinkEvent::Live(live) => LinkView {
            live: Some(live),
            ..view
        },
        LinkEvent::Catalog(catalog) => LinkView { catalog, ..view },
        LinkEvent::Disconnected => LinkView::default(),
        // The UI shows the state, not the event log.
        LinkEvent::Ack { .. }
        | LinkEvent::Event(_)
        | LinkEvent::EventsLost { .. }
        | LinkEvent::Outdated { .. }
        | LinkEvent::Quiet
        | LinkEvent::Recovered => view,
    }
}

#[cfg(test)]
mod tests {
    use protocol::message::Outcome;
    use protocol::{Catalog, Live, Phase};

    use super::*;

    fn playing() -> Live {
        Live {
            phase: Phase::Playing,
            position: 4.0,
            ..Live::default()
        }
    }

    #[test]
    fn connecting_then_a_live_state_shows_both() {
        let view = apply_event(
            LinkView::default(),
            LinkEvent::Connected {
                extension_version: "1".into(),
            },
        );
        let view = apply_event(view, LinkEvent::Live(playing()));
        assert_eq!(
            view.status,
            LinkStatus::Connected {
                extension_version: "1".into()
            }
        );
        assert_eq!(view.live, Some(playing()));
    }

    #[test]
    fn an_older_state_after_a_newer_one_is_ignored() {
        let newer = Live {
            sequence: 5,
            ..playing()
        };
        let older = Live {
            sequence: 4,
            position: 1.0,
            ..playing()
        };
        let view = apply_event(LinkView::default(), LinkEvent::Live(newer.clone()));
        assert_eq!(apply_event(view, LinkEvent::Live(older)).live, Some(newer));
    }

    #[test]
    fn a_catalog_is_kept_with_the_state() {
        let catalog = Catalog {
            revision: 3,
            ..Catalog::default()
        };
        let view = apply_event(
            LinkView::default(),
            LinkEvent::Catalog(Box::new(catalog.clone())),
        );
        let view = apply_event(view, LinkEvent::Live(playing()));
        assert_eq!(*view.catalog, catalog);
        assert_eq!(view.live, Some(playing()));
    }

    #[test]
    fn losing_the_link_forgets_the_old_state() {
        let view = LinkView {
            status: LinkStatus::Connected {
                extension_version: "1".into(),
            },
            live: Some(playing()),
            catalog: Box::default(),
        };
        assert_eq!(
            apply_event(view, LinkEvent::Disconnected),
            LinkView::default()
        );
    }

    #[test]
    fn an_ack_changes_nothing() {
        let view = LinkView {
            live: Some(playing()),
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
