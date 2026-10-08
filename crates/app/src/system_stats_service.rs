//! Keeps the latest machine stats for the window.

use std::sync::Mutex;

use protocol::SystemStats;

use crate::stats_source::StatsSource;

/// Reads the source on `refresh` and hands out the last reading.
pub struct SystemStatsService {
    source: Mutex<Box<dyn StatsSource>>,
    latest: Mutex<SystemStats>,
}

impl SystemStatsService {
    /// A service over `source`, with an empty first reading.
    pub fn new(source: impl StatsSource + 'static) -> Self {
        Self {
            source: Mutex::new(Box::new(source)),
            latest: Mutex::default(),
        }
    }

    /// Takes a new reading; the old one stays when the source is unusable.
    pub fn refresh(&self) {
        let Ok(mut source) = self.source.lock() else {
            return;
        };
        let stats = source.read();
        if let Ok(mut latest) = self.latest.lock() {
            *latest = stats;
        }
    }

    /// The last reading.
    pub fn latest(&self) -> SystemStats {
        self.latest
            .lock()
            .map(|latest| latest.clone())
            .unwrap_or_default()
    }
}
