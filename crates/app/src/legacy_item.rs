//! One song in a v1 setlist.

use serde::{Deserialize, Deserializer};

/// A v1 setlist item: v1 pointed at a region number and kept the name next to it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LegacyItem {
    /// The region name when the item was added.
    pub name: String,
    /// The region number v1 stored (as text or as a number), if it can be read.
    #[serde(default, rename = "regionId", deserialize_with = "region_number")]
    pub region_number: Option<u32>,
}

/// v1 wrote the number as a string in some places and as a number in others.
fn region_number<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u32>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Stored {
        Number(u32),
        Text(String),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Option::<Stored>::deserialize(deserializer)? {
        Some(Stored::Number(number)) => Some(number),
        Some(Stored::Text(text)) => text.trim().parse().ok(),
        Some(Stored::Other(_)) | None => None,
    })
}
