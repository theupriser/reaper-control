//! A stats source for tests.

use protocol::SystemStats;

use crate::stats_source::StatsSource;

/// Answers with the stats it was given.
pub struct FakeStatsSource {
    stats: SystemStats,
}

impl FakeStatsSource {
    /// A source that always answers `stats`.
    pub fn new(stats: SystemStats) -> Self {
        Self { stats }
    }
}

impl StatsSource for FakeStatsSource {
    fn read(&mut self) -> SystemStats {
        self.stats.clone()
    }
}
