use shared_kernel::SongId;

use crate::EntryId;

/// What an accepted edit did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetlistEvent {
    /// The setlist got a new name.
    Renamed {
        /// The new name.
        name: String,
    },
    /// An entry was added.
    EntryAdded {
        /// The new entry.
        entry: EntryId,
        /// Its song.
        song: SongId,
        /// Its index.
        at: usize,
    },
    /// An entry was removed.
    EntryRemoved {
        /// The removed entry.
        entry: EntryId,
    },
    /// An entry changed place.
    EntryMoved {
        /// The moved entry.
        entry: EntryId,
        /// Where it was.
        from: usize,
        /// Where it is now.
        to: usize,
    },
}
