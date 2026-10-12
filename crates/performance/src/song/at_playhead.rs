//! Which song of the setlist the playhead is in.

use shared_kernel::Seconds;

use crate::PlannedSong;

/// REAPER snaps a seek to a sample, so a playhead sent to a song's start can read a hair before it.
const START_TOLERANCE: f64 = 0.001;

/// The song whose window holds `position`. Where windows overlap, the song that starts last wins
/// (the playhead is in its region; a song with a custom length reaches into the next one), and
/// when several entries share that start the current one is kept. A playhead a millisecond
/// before a start counts as inside that song, so a song that begins where the one before ends can
/// be jumped to.
pub(crate) fn song_at_playhead(
    songs: &[PlannedSong],
    current: Option<usize>,
    position: Seconds,
) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (index, song) in songs.iter().enumerate() {
        let start = song.window.start().get();
        let reached =
            position.get() + START_TOLERANCE >= start && position.get() <= song.window.end().get();
        if !reached {
            continue;
        }
        let better = best.is_none_or(|(held, held_start)| {
            start > held_start || (start == held_start && current == Some(index) && held != index)
        });
        if better {
            best = Some((index, start));
        }
    }
    best.map(|(index, _)| index)
}
