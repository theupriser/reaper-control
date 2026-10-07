/// Whether the numbers on screen can be trusted as live (SPEC S-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    /// The last update is recent.
    Fresh,
    /// No update yet, or the heartbeat is overdue: do not show as live.
    Stale,
}
