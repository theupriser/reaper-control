use shared_kernel::Seconds;

use crate::{Freshness, LiveFeed, PerformanceView};

/// Everything the Player screen shows: the performance, the playhead, and
/// whether it is live.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerView {
    /// Sequence number of the update this is built from.
    pub seq: Option<u64>,
    /// Whether the numbers may be shown as live.
    pub freshness: Freshness,
    /// Where the playhead is.
    pub position: Seconds,
    /// The performance.
    pub performance: PerformanceView,
}

impl PlayerView {
    /// Combines the performance view with the feed's state at `now`.
    pub fn new(
        performance: PerformanceView,
        position: Seconds,
        feed: &LiveFeed,
        now: Seconds,
    ) -> Self {
        Self {
            seq: feed.seq(),
            freshness: feed.freshness(now),
            position,
            performance,
        }
    }
}
