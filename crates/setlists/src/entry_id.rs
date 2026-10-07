/// Identity of one entry. Unique inside a setlist and never reused there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntryId(u64);

impl EntryId {
    /// Wraps a raw id (as stored).
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw id.
    pub fn get(self) -> u64 {
        self.0
    }
}
