//! A v1 setlist mapped onto the songs of a project.

use protocol::SetlistInfo;

use crate::legacy_item::LegacyItem;

/// The outcome of mapping one v1 setlist.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSetlist {
    /// The setlist with the items that found their song.
    pub setlist: SetlistInfo,
    /// Items that found no song: offered for repair, never dropped silently (ADR-008).
    pub unresolved: Vec<LegacyItem>,
}
