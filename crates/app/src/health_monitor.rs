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

mod state;
use state::State;

/// How long a lost link is only "lost" before the app names a cause.
const GRACE: Duration = Duration::from_secs(5);

/// How often the operating system is asked whether REAPER runs.
const PROCESS_CHECK_EVERY: Duration = Duration::from_secs(5);

/// Turns link events and the passing of time into `LinkHealth`.
pub struct HealthMonitor {
    state: Mutex<State>,
    events: Arc<EventBus>,
    clock: Arc<dyn Clock>,
    processes: Arc<dyn ProcessCheck>,
}

impl HealthMonitor {
    /// A monitor that starts out lost, as the app has not connected yet. The start is not
    /// announced: read `health()` for the first value.
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
                last_process_check: None,
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
    /// The operating system is asked outside the lock and at most every few seconds.
    pub fn tick(&self) {
        let now = self.clock.now();
        let Some(before) = self.due_for_process_check(now) else {
            return;
        };
        let running = self.processes.reaper_is_running();
        let changed = {
            let Ok(mut state) = self.state.lock() else {
                return;
            };
            if state.health != before {
                return;
            }
            let next = match (&before, running) {
                (LinkHealth::Dead(LinkCause::ExtensionOutdated { .. }), true) => return,
                (_, true) => LinkHealth::Dead(LinkCause::ExtensionNotLoaded),
                (_, false) => LinkHealth::Dead(LinkCause::ReaperNotRunning),
            };
            Self::change(&mut state, next)
        };
        self.announce(changed);
    }

    /// The health to re-check against, when a cause has to be found or looked at again.
    fn due_for_process_check(&self, now: Duration) -> Option<LinkHealth> {
        let mut state = self.state.lock().ok()?;
        let wanted = match state.health {
            LinkHealth::Lost => now.saturating_sub(state.lost_since) >= GRACE,
            LinkHealth::Dead(_) => true,
            LinkHealth::Connected | LinkHealth::Degraded => false,
        };
        let recent = state
            .last_process_check
            .is_some_and(|last| now.saturating_sub(last) < PROCESS_CHECK_EVERY);
        if !wanted || recent {
            return None;
        }
        state.last_process_check = Some(now);
        Some(state.health.clone())
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
