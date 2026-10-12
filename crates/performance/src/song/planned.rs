use shared_kernel::Seconds;

use crate::SongWindow;

/// One entry of the setlist as the performance sees it. The extension builds
/// it from the catalogue (window with `!length` applied, `!1008`).
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedSong {
    /// The song's identity (region GUID).
    pub song_id: String,
    /// Where it plays.
    pub window: SongWindow,
    /// Whether playback halts at the end (`!1008`).
    pub hard_stop: bool,
    /// Where the `!1008` marker lies. REAPER can stop there by itself (SWS); the performance then
    /// keeps time up to the end of the window.
    pub hard_stop_marker: Option<Seconds>,
}
