/// Song identity: the region GUID (ADR-008). Opaque to everything but equality.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SongId(String);

impl SongId {
    /// Wraps a GUID as REAPER reports it.
    pub fn new(guid: impl Into<String>) -> Self {
        Self(guid.into())
    }

    /// The GUID text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
