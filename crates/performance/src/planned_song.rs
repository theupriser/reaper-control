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
}
