//! What happens to each thing the link reports: health, announcements, the view, the window.

use std::sync::{Arc, Mutex};

use link::LinkEvent;
use protocol::LinkView;

use crate::apply_event::apply_event;
use crate::event_bus::EventBus;
use crate::health_monitor::HealthMonitor;
use crate::link_session::LinkSession;

/// Takes link events one by one, from the real link and from the simulator alike.
pub struct LinkPipeline {
    session: LinkSession,
    view: Arc<Mutex<LinkView>>,
    events: Arc<EventBus>,
    health: Arc<HealthMonitor>,
    on_change: Box<dyn Fn(LinkView) + Send>,
}

impl LinkPipeline {
    /// A pipeline that publishes on `events`, keeps the view in `view` and calls `on_change`
    /// with the new view after every event.
    pub fn new(
        view: Arc<Mutex<LinkView>>,
        events: Arc<EventBus>,
        health: Arc<HealthMonitor>,
        on_change: impl Fn(LinkView) + Send + 'static,
    ) -> Self {
        Self {
            session: LinkSession::default(),
            view,
            events,
            health,
            on_change: Box::new(on_change),
        }
    }

    /// Handles one event. Returns false when the view can no longer be updated.
    pub fn deliver(&mut self, event: LinkEvent) -> bool {
        self.health.observe(&event);
        if let Some(announcement) = self.session.observe(&event) {
            self.events.publish(&announcement);
        }
        let Ok(mut guard) = self.view.lock() else {
            return false;
        };
        *guard = apply_event(guard.clone(), event);
        (self.on_change)(guard.clone());
        true
    }
}
