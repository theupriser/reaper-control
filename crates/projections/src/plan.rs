use catalogue::{Cue, Song};
use performance::{PlannedSong, SongWindow};
use setlists::{EntryId, Setlist};

/// The songs a setlist will play, in order, ready for the `Performance`.
/// Entries whose song is gone are left out and listed in `skipped`.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// What to play, in order.
    pub songs: Vec<PlannedSong>,
    /// Entries that could not be planned.
    pub skipped: Vec<EntryId>,
}

impl Plan {
    /// Builds the plan: each window ends at the region end or at the start
    /// plus `!length`, and `!1008` makes the song a hard stop.
    pub fn build(setlist: &Setlist, songs: &[Song], cues: &[Cue]) -> Self {
        let mut plan = Self {
            songs: Vec::new(),
            skipped: Vec::new(),
        };
        for entry in setlist.entries() {
            match songs.iter().find(|song| song.id() == entry.song()) {
                Some(song) => match planned(song, cues) {
                    Some(planned) => plan.songs.push(planned),
                    None => plan.skipped.push(entry.id()),
                },
                None => plan.skipped.push(entry.id()),
            }
        }
        plan
    }
}

fn planned(song: &Song, cues: &[Cue]) -> Option<PlannedSong> {
    let found = song.directives(cues);
    let end = shared_kernel::Seconds::new(song.start().get() + song.length(&found).get()).ok()?;
    Some(PlannedSong {
        song_id: song.id().as_str().to_string(),
        window: SongWindow::new(song.start(), end).ok()?,
        hard_stop: found.hard_stop(),
        hard_stop_marker: found.hard_stop_at(),
    })
}
