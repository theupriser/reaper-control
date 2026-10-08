//! Mapping v1 items onto songs (ADR-008, step 7).

use protocol::{EntryInfo, SetlistInfo, SongInfo};

use crate::legacy_setlist::LegacySetlist;
use crate::resolved_setlist::ResolvedSetlist;

/// Finds the song for every item by exact name. When several songs share a name they are used in
/// timeline order, one per item, so a setlist that played a song twice keeps both entries.
///
/// The region number v1 stored is not in the catalog yet, so the name is the only key here.
#[must_use]
pub fn resolve(legacy: &LegacySetlist, songs: &[SongInfo]) -> ResolvedSetlist {
    let mut used = vec![false; songs.len()];
    let mut entries = Vec::new();
    let mut unresolved = Vec::new();
    for item in &legacy.items {
        let found = songs
            .iter()
            .zip(used.iter_mut())
            .find(|(song, taken)| !**taken && song.name == item.name);
        match found {
            Some((song, taken)) => {
                *taken = true;
                entries.push(EntryInfo {
                    id: entries.len() as u64,
                    song_id: song.id.clone(),
                });
            }
            None => unresolved.push(item.clone()),
        }
    }
    ResolvedSetlist {
        setlist: SetlistInfo {
            id: legacy.id.clone(),
            name: legacy.name.clone(),
            revision: 0,
            entries,
        },
        unresolved,
    }
}
