//! Mapping v1 items onto songs (ADR-008, step 7).

use protocol::{EntryInfo, SetlistInfo, SongInfo};

use crate::legacy_setlist::LegacySetlist;
use crate::resolved_setlist::ResolvedSetlist;

/// Finds the song for every item: first a song with the same name and region number, then one with
/// the same name. When several songs share a name they are used in timeline order, one per item,
/// so a setlist that played a song twice keeps both entries. The number alone is not trusted: a
/// renamed or renumbered region is reported as unresolved rather than guessed.
#[must_use]
pub fn resolve(legacy: &LegacySetlist, songs: &[SongInfo]) -> ResolvedSetlist {
    let mut used = vec![false; songs.len()];
    let mut entries = Vec::new();
    let mut unresolved = Vec::new();
    for item in &legacy.items {
        let free_with_name = |index: &usize| {
            let free = !used.get(*index).copied().unwrap_or(true);
            free && songs.get(*index).is_some_and(|song| song.name == item.name)
        };
        let found = (0..songs.len())
            .find(|index| {
                free_with_name(index)
                    && songs.get(*index).map(|song| song.number) == item.region_number
            })
            .or_else(|| (0..songs.len()).find(free_with_name));
        match found.and_then(|index| songs.get(index).map(|song| (index, song))) {
            Some((index, song)) => {
                if let Some(taken) = used.get_mut(index) {
                    *taken = true;
                }
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
