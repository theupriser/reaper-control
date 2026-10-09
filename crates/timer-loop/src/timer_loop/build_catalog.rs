use catalogue::{Cue, MarkerName, Song};
use performance::PlannedSong;
use protocol::{Catalog, CueInfo, SongInfo};
use reaper_port::{Marker, Region};

use super::project_setlists::ProjectSetlists;

/// The catalog the app is shown. Its songs are the planned songs, in the same order, so the index
/// the app is told (`Live::current_song`) points at the same song in both. Markers that only
/// carry commands are not cues.
pub(super) fn build_catalog(
    revision: u64,
    setlist_revision: u64,
    project_setlists: &ProjectSetlists,
    planned: &[PlannedSong],
    regions: &[Region],
    markers: &[Marker],
) -> Catalog {
    let cues: Vec<Cue> = markers
        .iter()
        .map(|marker| Cue::new(marker.name.clone(), marker.position))
        .collect();
    Catalog {
        revision,
        setlist_revision,
        project_id: project_setlists
            .project_id()
            .unwrap_or_default()
            .to_string(),
        songs: planned
            .iter()
            .filter_map(|song| regions.iter().find(|r| r.id.as_str() == song.song_id))
            .filter_map(|region| song_info(region, &cues))
            .collect(),
        project_songs: regions
            .iter()
            .filter_map(|region| song_info(region, &cues))
            .collect(),
        cues: markers
            .iter()
            .enumerate()
            .filter(|(_, marker)| !MarkerName::parse(&marker.name).is_hidden())
            .map(|(index, marker)| CueInfo {
                id: format!("cue-{index}"),
                name: marker.name.clone(),
                position: marker.position.get(),
            })
            .collect(),
        setlists: project_setlists.setlists().to_vec(),
        active_setlist: project_setlists.active().map(str::to_string),
    }
}

fn song_info(region: &Region, cues: &[Cue]) -> Option<SongInfo> {
    let song = Song::new(
        region.id.clone(),
        region.name.clone(),
        region.start,
        region.end,
    )
    .ok()?;
    let directives = song.directives(cues);
    Some(SongInfo {
        id: region.id.as_str().to_string(),
        number: region.number,
        name: region.name.clone(),
        start: region.start.get(),
        end: region.end.get(),
        colour: None,
        hard_stop: directives.hard_stop(),
        length: directives.length().map(|length| length.get()),
        bpm: directives.tempo().map(|tempo| tempo.get()),
    })
}
