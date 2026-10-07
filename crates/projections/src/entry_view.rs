use setlists::EntryId;
use shared_kernel::{Seconds, SongId};

/// One row of the setlist as shown. A row whose song is gone keeps its place
/// and is marked, never dropped.
#[derive(Debug, Clone, PartialEq)]
pub struct EntryView {
    /// The entry's identity.
    pub entry: EntryId,
    /// The song it points at.
    pub song: SongId,
    /// The song's name, `None` when the project no longer has it.
    pub name: Option<String>,
    /// The playing length (`!length` applied), `None` when the song is gone.
    pub length: Option<Seconds>,
    /// The song ends with a hard stop.
    pub hard_stop: bool,
}

impl EntryView {
    /// The song is gone from the project.
    pub fn is_dangling(&self) -> bool {
        self.name.is_none()
    }
}
