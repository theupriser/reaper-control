//! One song in a v1 setlist.

use serde::Deserialize;

/// A v1 setlist item: v1 pointed at a region number and kept the name next to it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LegacyItem {
    /// The region name when the item was added.
    pub name: String,
}
