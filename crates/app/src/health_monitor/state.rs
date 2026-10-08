use std::time::Duration;

use crate::link_health::LinkHealth;

/// What the monitor remembers between events.
pub(super) struct State {
    pub(super) health: LinkHealth,
    pub(super) lost_since: Duration,
    pub(super) last_process_check: Option<Duration>,
}
