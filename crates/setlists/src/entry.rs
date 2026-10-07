use shared_kernel::SongId;

use crate::EntryId;

/// One position in a setlist: a reference to a song.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    id: EntryId,
    song: SongId,
}

impl Entry {
    /// An entry for a song.
    pub fn new(id: EntryId, song: SongId) -> Self {
        Self { id, song }
    }

    /// The entry's identity.
    pub fn id(&self) -> EntryId {
        self.id
    }

    /// The song it points at.
    pub fn song(&self) -> &SongId {
        &self.song
    }
}
