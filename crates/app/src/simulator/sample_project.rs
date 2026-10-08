//! The project the simulator plays.

use performance::{TempoMap, TimeSignature};
use reaper_port::{FakeReaper, ReaperPort, Region};
use shared_kernel::{Bpm, Seconds, SongId};

fn seconds(value: f64) -> Seconds {
    Seconds::new(value).unwrap_or(Seconds::ZERO)
}

fn region(id: &str, name: &str, start: f64, end: f64) -> Region {
    Region {
        id: SongId::new(id),
        name: name.to_string(),
        start: seconds(start),
        end: seconds(end),
    }
}

/// Two songs back to back, Song A (20 s) and Song B (25 s), like the sample project in `.dev`.
/// It already has its project id, as a saved project does, so every run is the same.
#[must_use]
pub fn sample_project() -> FakeReaper {
    let mut reaper = FakeReaper::new(
        vec![
            region("song-a", "Song A", 0.0, 20.0),
            region("song-b", "Song B", 20.0, 45.0),
        ],
        Vec::new(),
        TempoMap::constant(Bpm::FALLBACK, TimeSignature::common()),
    );
    reaper.set_ext_state("RC2", "project_id", "sample-project");
    reaper
}
