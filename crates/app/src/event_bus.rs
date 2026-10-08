//! In-process delivery of app events.

use std::sync::Mutex;

use crate::app_event::AppEvent;

type Subscriber = Box<dyn Fn(&AppEvent) + Send + Sync>;

/// In-process delivery of `AppEvent`s to everyone who subscribed, in the order they were published.
#[derive(Default)]
pub struct EventBus {
    subscribers: Mutex<Vec<Subscriber>>,
}

impl EventBus {
    /// `subscriber` is called for every event published from now on.
    pub fn subscribe(&self, subscriber: impl Fn(&AppEvent) + Send + Sync + 'static) {
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.push(Box::new(subscriber));
        }
    }

    /// Delivers `event` to every subscriber.
    pub fn publish(&self, event: &AppEvent) {
        if let Ok(subscribers) = self.subscribers.lock() {
            for subscriber in subscribers.iter() {
                subscriber(event);
            }
        }
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("EventBus").finish_non_exhaustive()
    }
}
