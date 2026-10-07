use shared_kernel::SongId;

use crate::EntryId;

/// A change to a setlist. The only way to change one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Give the setlist another name.
    Rename(String),
    /// Add a song at an index, or at the end.
    Add {
        /// The song to add.
        song: SongId,
        /// Where to put it; `None` appends.
        at: Option<usize>,
    },
    /// Take an entry out.
    Remove(EntryId),
    /// Put an entry at an index of the resulting list.
    Move {
        /// The entry to move.
        entry: EntryId,
        /// Its index after the move.
        to: usize,
    },
}
