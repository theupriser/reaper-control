use catalogue::{Cue, Song};
use setlists::{Revision, Setlist, SetlistId};

use crate::EntryView;

/// A setlist ready to show, resolved against the catalogue.
#[derive(Debug, Clone, PartialEq)]
pub struct SetlistView {
    /// The setlist's identity.
    pub id: SetlistId,
    /// Its name.
    pub name: String,
    /// The revision to pass back when editing.
    pub revision: Revision,
    /// The rows, in order.
    pub entries: Vec<EntryView>,
}

impl SetlistView {
    /// Resolves every entry against the songs and cues of the project.
    pub fn of(setlist: &Setlist, songs: &[Song], cues: &[Cue]) -> Self {
        let entries = setlist
            .entries()
            .iter()
            .map(|entry| {
                let song = songs.iter().find(|song| song.id() == entry.song());
                let found = song.map(|song| (song, song.directives(cues)));
                EntryView {
                    entry: entry.id(),
                    song: entry.song().clone(),
                    name: found.as_ref().map(|(song, _)| song.name().to_string()),
                    length: found.as_ref().map(|(song, found)| song.length(found)),
                    hard_stop: found.is_some_and(|(_, found)| found.hard_stop()),
                }
            })
            .collect();
        Self {
            id: setlist.id().clone(),
            name: setlist.name().to_string(),
            revision: setlist.revision(),
            entries,
        }
    }
}
