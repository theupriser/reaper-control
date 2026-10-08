//! Follows the link's health and announces every change.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use link::LinkEvent;

use crate::app_event::AppEvent;
use crate::clock::Clock;
use crate::event_bus::EventBus;
use crate::link_cause::LinkCause;
use crate::link_health::LinkHealth;
use crate::process_check::ProcessCheck;

/// How long a lost link is only "lost" before the app names a cause.
const GRACE: Duration = Duration::from_secs(5);

struct State {
    health: LinkHealth,
    lost_since: Duration,
}

/// Turns link events and the passing of time into `LinkHealth`.
pub struct HealthMonitor {
    state: Mutex<State>,
    events: Arc<EventBus>,
    clock: Arc<dyn Clock>,
    processes: Arc<dyn ProcessCheck>,
}

impl HealthMonitor {
    /// A monitor that starts out lost, as the app has not connected yet.
    pub fn new(
        events: Arc<EventBus>,
        clock: Arc<dyn Clock>,
        processes: Arc<dyn ProcessCheck>,
    ) -> Self {
        let lost_since = clock.now();
        Self {
            state: Mutex::new(State {
                health: LinkHealth::Lost,
                lost_since,
            }),
            events,
            clock,
            processes,
        }
    }

    /// The current health.
    #[must_use]
    pub fn health(&self) -> LinkHealth {
        self.state
            .lock()
            .map(|state| state.health.clone())
            .unwrap_or(LinkHealth::Lost)
    }

    /// Takes in what the link reported.
    pub fn observe(&self, event: &LinkEvent) {
        let now = self.clock.now();
        let changed = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            let next = match (event, &state.health) {
                (LinkEvent::Connected { .. } | LinkEvent::Recovered, _) => LinkHealth::Connected,
                (LinkEvent::Quiet, LinkHealth::Connected) => LinkHealth::Degraded,
                (LinkEvent::Outdated { found }, _) => {
                    LinkHealth::Dead(LinkCause::ExtensionOutdated { found: *found })
                }
                (LinkEvent::Disconnected, LinkHealth::Connected | LinkHealth::Degraded) => {
                    state.lost_since = now;
                    LinkHealth::Lost
                }
                _ => return,
            };
            Self::change(&mut state, next)
        };
        self.announce(changed);
    }

    /// Lets time pass: a link that stays lost gets a cause, and a cause is looked at again.
    pub fn tick(&self) {
        let now = self.clock.now();
        let changed = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            let waiting = match state.health {
                LinkHealth::Lost => now.saturating_sub(state.lost_since) >= GRACE,
                LinkHealth::Dead(LinkCause::ReaperNotRunning | LinkCause::ExtensionNotLoaded) => {
                    true
                }
                _ => false,
            };
            if !waiting {
                return;
            }
            let cause = if self.processes.reaper_is_running() {
                LinkCause::ExtensionNotLoaded
            } else {
                LinkCause::ReaperNotRunning
            };
            Self::change(&mut state, LinkHealth::Dead(cause))
        };
        self.announce(changed);
    }

    fn change(state: &mut State, next: LinkHealth) -> Option<LinkHealth> {
        if state.health == next {
            return None;
        }
        state.health = next.clone();
        Some(next)
    }

    fn announce(&self, changed: Option<LinkHealth>) {
        if let Some(health) = changed {
            self.events.publish(&AppEvent::LinkHealthChanged { health });
        }
    }
}

#[cfg(test)]
mod tests;
