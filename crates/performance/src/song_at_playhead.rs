//! Which song of the setlist the playhead is in.

use shared_kernel::Seconds;

use crate::PlannedSong;

/// The song whose window holds `position`. Where windows overlap, the song that starts last wins
/// (the playhead is in its region; a song with a custom length reaches into the next one), and
/// when several entries share that start the current one is kept.
pub(crate) fn song_at_playhead(
    songs: &[PlannedSong],
    current: Option<usize>,
    position: Seconds,
) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (index, song) in songs.iter().enumerate() {
        if !song.window.contains(position) {
            continue;
        }
        let start = song.window.start().get();
        let better = best.is_none_or(|(held, held_start)| {
            start > held_start || (start == held_start && current == Some(index) && held != index)
        });
        if better {
            best = Some((index, start));
        }
    }
    best.map(|(index, _)| index)
}
