use catalogue::{Cue, Song};
use performance::{PlannedSong, SongWindow};
use reaper_port::{Marker, Region};
use shared_kernel::Seconds;

use super::project_setlists::ProjectSetlists;

/// The songs of the project as the performance plays them: every region in timeline order (or the
/// order of the played setlist), with `!length` applied to its window and `!1008` as its hard stop.
/// A region that cannot be a song (no length) is left out.
pub(super) fn plan_songs(
    regions: &[Region],
    markers: &[Marker],
    project_setlists: &ProjectSetlists,
) -> Vec<PlannedSong> {
    let cues: Vec<Cue> = markers
        .iter()
        .map(|marker| Cue::new(marker.name.clone(), marker.position))
        .collect();
    let timeline = regions
        .iter()
        .filter_map(|region| planned(region, &cues))
        .collect();
    project_setlists.arrange(timeline)
}

fn planned(region: &Region, cues: &[Cue]) -> Option<PlannedSong> {
    let song = Song::new(
        region.id.clone(),
        region.name.clone(),
        region.start,
        region.end,
    )
    .ok()?;
    let directives = song.directives(cues);
    let end = Seconds::new(song.start().get() + song.length(&directives).get()).ok()?;
    Some(PlannedSong {
        song_id: song.id().as_str().to_string(),
        window: SongWindow::new(song.start(), end).ok()?,
        hard_stop: directives.hard_stop(),
    })
}
