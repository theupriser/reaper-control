use performance::TempoMap;
use shared_kernel::Seconds;

use crate::fake_project_state::FakeProjectState;
use crate::{Marker, ReaperPort, Region, Transport};

/// How long REAPER 7.82 holds the playhead for two bars of 4/4 at 120 bpm (measured: the playhead
/// stood still for about 4 s, then moved).
const COUNT_IN: f64 = 4.0;

/// A REAPER that does exactly what it is told and nothing else: the playhead
/// moves at normal speed while playing, and only when `advance` is called, so
/// every run gives the same result. Like REAPER it counts in (holds the playhead on the spot) when
/// playback starts from a stop or pause with the count-in armed, never on a jump while playing;
/// switching the count-in off during the count-in does not shorten it.
#[derive(Debug, Clone)]
pub struct FakeReaper {
    now: Seconds,
    position: Seconds,
    transport: Transport,
    count_in: bool,
    count_in_left: f64,
    regions: Vec<Region>,
    markers: Vec<Marker>,
    // Boxed to keep the struct under 128 bytes.
    tempo_map: Box<TempoMap>,
    project: Box<FakeProjectState>,
}

impl FakeReaper {
    /// A stopped project with the playhead at zero.
    pub fn new(regions: Vec<Region>, markers: Vec<Marker>, tempo_map: TempoMap) -> Self {
        Self {
            now: Seconds::ZERO,
            position: Seconds::ZERO,
            transport: Transport::Stopped,
            count_in: false,
            count_in_left: 0.0,
            regions,
            markers,
            tempo_map: Box::new(tempo_map),
            project: Box::default(),
        }
    }

    /// Edits the project the way a user does: the regions are replaced and the change count moves.
    pub fn replace_regions(&mut self, regions: Vec<Region>) {
        self.regions = regions;
        self.project.change_count += 1;
    }

    /// The user switches to another project tab: its regions and markers replace the current ones,
    /// it has its own (empty) ExtState, the transport of that tab is stopped, and the change count
    /// is left alone, because two projects can have the same one.
    pub fn switch_project(&mut self, regions: Vec<Region>, markers: Vec<Marker>) {
        self.regions = regions;
        self.markers = markers;
        self.project.ext_state.clear();
        self.project.project_path = None;
        self.transport = Transport::Stopped;
        self.position = Seconds::ZERO;
        self.count_in_left = 0.0;
        self.project.project_token += 1;
    }

    /// Saves the project under a path, or moves or copies the file there: the ExtState stays with
    /// the file, so a copy carries the original's values.
    pub fn set_project_path(&mut self, path: Option<&str>) {
        self.project.project_path = path.map(str::to_string);
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
            let held = span.min(self.count_in_left);
            self.count_in_left -= held;
            let moved = self.position.get() + span - held;
            self.position = Seconds::new(moved).unwrap_or(self.position);
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
        self.project.change_count
    }

    fn project_token(&self) -> u64 {
        self.project.project_token
    }

    fn project_path(&self) -> Option<String> {
        self.project.project_path.clone()
    }

    fn regions(&self) -> Vec<Region> {
        self.regions.clone()
    }

    fn markers(&self) -> Vec<Marker> {
        self.markers.clone()
    }

    fn tempo_map(&self) -> TempoMap {
        (*self.tempo_map).clone()
    }

    fn ext_state(&self, section: &str, key: &str) -> Option<String> {
        self.project
            .ext_state
            .get(&(section.to_string(), key.to_string()))
            .cloned()
    }

    fn play(&mut self) {
        if self.transport != Transport::Playing && self.count_in {
            self.count_in_left = COUNT_IN;
        }
        self.transport = Transport::Playing;
    }

    fn pause(&mut self) {
        if self.transport == Transport::Playing {
            self.transport = Transport::Paused;
            self.count_in_left = 0.0;
        }
    }

    fn seek(&mut self, position: Seconds) {
        self.position = position;
    }

    fn set_count_in(&mut self, enabled: bool) {
        self.count_in = enabled;
    }

    fn set_ext_state(&mut self, section: &str, key: &str, value: &str) {
        self.project
            .ext_state
            .insert((section.to_string(), key.to_string()), value.to_string());
    }
}
