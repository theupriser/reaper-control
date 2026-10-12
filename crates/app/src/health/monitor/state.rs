use std::time::Duration;

use app_core::LinkHealth;

/// What the monitor remembers between events.
pub(super) struct State {
    pub(super) health: LinkHealth,
    pub(super) lost_since: Duration,
    pub(super) last_process_check: Option<Duration>,
}
