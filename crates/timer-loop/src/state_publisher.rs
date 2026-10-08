use link::LinkServer;
use protocol::{Live, WireEvent};
use reaper_port::ReaperPort;

use crate::TimerLoop;

/// A quiet state is pushed again after this many seconds, so the app can tell a calm extension
/// from a stuck one.
const HEARTBEAT_SECONDS: f64 = 1.0;

/// Pushes what the timer loop reports to the link server: new events at once, the catalog when
/// its revisions change, the live state when it changes or the heartbeat is due.
#[derive(Debug, Default)]
pub struct StatePublisher {
    published: Option<Live>,
    sequence: u64,
    published_at: f64,
    published_revisions: Option<(u64, u64)>,
}

impl StatePublisher {
    /// Pushes everything new. `on_event` sees each event just before it goes out.
    pub fn publish<Port: ReaperPort>(
        &mut self,
        timer_loop: &mut TimerLoop<Port>,
        server: &LinkServer,
        mut on_event: impl FnMut(&WireEvent),
    ) {
        for event in timer_loop.take_events() {
            on_event(&event);
            server.publish_event(event);
        }
        let catalog = timer_loop.catalog();
        let revisions = (catalog.revision, catalog.setlist_revision);
        if self.published_revisions != Some(revisions) {
            server.publish_catalog(catalog.clone());
            self.published_revisions = Some(revisions);
        }
        let live = timer_loop.live();
        let now = timer_loop.now();
        if self.published.as_ref() != Some(&live) || now - self.published_at >= HEARTBEAT_SECONDS {
            self.sequence += 1;
            self.published = Some(live.clone());
            self.published_at = now;
            server.publish(Live {
                sequence: self.sequence,
                timestamp: now,
                ..live
            });
        }
    }
}
