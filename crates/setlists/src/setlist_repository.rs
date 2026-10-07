use crate::{Revision, SaveError, Setlist, SetlistId};

/// Where setlists are kept. Saves are optimistic: the caller names the
/// revision it started from (`None` for a setlist that must not exist yet).
pub trait SetlistRepository {
    /// One setlist.
    fn load(&self, id: &SetlistId) -> Option<Setlist>;

    /// All setlists, ordered by id.
    fn list(&self) -> Vec<Setlist>;

    /// Stores a setlist if the stored one is still at `expected`.
    fn save(&mut self, setlist: Setlist, expected: Option<Revision>) -> Result<(), SaveError>;

    /// Removes a setlist if it is still at `expected`.
    fn delete(&mut self, id: &SetlistId, expected: Revision) -> Result<(), SaveError>;
}
