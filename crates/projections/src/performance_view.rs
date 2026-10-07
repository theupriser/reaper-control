use performance::{Flags, Performance, Phase};

/// What the Performance looks like from outside: the phase, where in the
/// setlist it is, and the settings. The UI renders it and never infers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformanceView {
    /// The phase, the only phase vocabulary.
    pub phase: Phase,
    /// The song playing or about to play.
    pub current: Option<String>,
    /// The song after it, if any.
    pub next: Option<String>,
    /// Zero-based place of the current song in the setlist.
    pub index: Option<usize>,
    /// The playback settings.
    pub flags: Flags,
}

impl PerformanceView {
    /// Reads the view off a performance, given the setlist's songs in order.
    pub fn of(performance: &Performance, songs: &[performance::PlannedSong]) -> Self {
        let index = performance.current_index();
        let id_at = |at: Option<usize>| {
            at.and_then(|at| songs.get(at))
                .map(|song| song.song_id.clone())
        };
        Self {
            phase: performance.phase(),
            current: id_at(index),
            next: id_at(index.and_then(|at| at.checked_add(1))),
            index,
            flags: performance.flags(),
        }
    }
}
