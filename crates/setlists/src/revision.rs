/// Version of a setlist. Rises by one per accepted edit; saves name the one they expect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(u64);

impl Revision {
    /// The revision of a new setlist.
    pub const INITIAL: Revision = Revision(0);

    /// Wraps a raw revision (as stored).
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw revision.
    pub fn get(self) -> u64 {
        self.0
    }

    /// The revision after one more accepted edit.
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}
