use shared_kernel::{Seconds, SongId};

/// A region of the project as REAPER reports it.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// Stable identity (the region GUID).
    pub id: SongId,
    /// The region's name.
    pub name: String,
    /// Where it starts.
    pub start: Seconds,
    /// Where it ends.
    pub end: Seconds,
}
