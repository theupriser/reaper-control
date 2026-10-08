use std::collections::BTreeMap;

use performance::TempoMap;
use shared_kernel::Seconds;

use crate::{Marker, ReaperPort, Region, Transport};

/// A REAPER that does exactly what it is told and nothing else: the playhead
/// moves at normal speed while playing, and only when `advance` is called, so
/// every run gives the same result.
#[derive(Debug, Clone)]
pub struct FakeReaper {
    now: Seconds,
    position: Seconds,
    transport: Transport,
    count_in: bool,
    regions: Vec<Region>,
    markers: Vec<Marker>,
    tempo_map: TempoMap,
    ext_state: BTreeMap<(String, String), String>,
    change_count: u64,
}

impl FakeReaper {
    /// A stopped project with the playhead at zero.
    pub fn new(regions: Vec<Region>, markers: Vec<Marker>, tempo_map: TempoMap) -> Self {
        Self {
            now: Seconds::ZERO,
            position: Seconds::ZERO,
            transport: Transport::Stopped,
            count_in: false,
            regions,
            markers,
            tempo_map,
            ext_state: BTreeMap::new(),
            change_count: 0,
        }
    }

    /// Edits the project the way a user does: the regions are replaced and the change count moves.
    pub fn replace_regions(&mut self, regions: Vec<Region>) {
        self.regions = regions;
        self.change_count += 1;
    }

    /// Lets time pass. The clock always moves forward; the playhead moves with
    /// it while playing. Zero, negative and non-finite spans do nothing.
    pub fn advance(&mut self, span: Seconds) {
        let span = span.get();
        if span <= 0.0 {
            return;
        }
        self.now = Seconds::new(self.now.get() + span).unwrap_or(self.now);
        if self.transport == Transport::Playing {
            self.position = Seconds::new(self.position.get() + span).unwrap_or(self.position);
        }
    }
}

impl ReaperPort for FakeReaper {
    fn now(&self) -> Seconds {
        self.now
    }

    fn position(&self) -> Seconds {
        self.position
    }

    fn transport(&self) -> Transport {
        self.transport
    }

    fn count_in(&self) -> bool {
        self.count_in
    }

    fn change_count(&self) -> u64 {
        self.change_count
    }

    fn regions(&self) -> Vec<Region> {
        self.regions.clone()
    }

    fn markers(&self) -> Vec<Marker> {
        self.markers.clone()
    }

    fn tempo_map(&self) -> TempoMap {
        self.tempo_map.clone()
    }

    fn ext_state(&self, section: &str, key: &str) -> Option<String> {
        self.ext_state
            .get(&(section.to_string(), key.to_string()))
            .cloned()
    }

    fn play(&mut self) {
        self.transport = Transport::Playing;
    }

    fn pause(&mut self) {
        if self.transport == Transport::Playing {
            self.transport = Transport::Paused;
        }
    }

    fn seek(&mut self, position: Seconds) {
        self.position = position;
    }

    fn set_count_in(&mut self, enabled: bool) {
        self.count_in = enabled;
    }

    fn set_ext_state(&mut self, section: &str, key: &str, value: &str) {
        self.ext_state
            .insert((section.to_string(), key.to_string()), value.to_string());
        self.change_count += 1;
    }
}
