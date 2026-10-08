//! A setlist as v1 stored it.

use serde::Deserialize;

use crate::legacy_item::LegacyItem;

/// A v1 setlist. Items come in playing order (v1 also stored a position; the list order wins).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LegacySetlist {
    /// v1's random id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// The songs in order.
    #[serde(default)]
    pub items: Vec<LegacyItem>,
}
