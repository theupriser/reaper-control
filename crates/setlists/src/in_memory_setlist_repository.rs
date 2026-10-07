use std::collections::BTreeMap;

use crate::{Revision, SaveError, Setlist, SetlistId, SetlistRepository};

/// A repository in memory: the test double, and the app's working copy.
#[derive(Debug, Clone, Default)]
pub struct InMemorySetlistRepository {
    stored: BTreeMap<SetlistId, Setlist>,
}

impl InMemorySetlistRepository {
    /// An empty repository.
    pub fn new() -> Self {
        Self::default()
    }

    fn stored_rev(&self, id: &SetlistId) -> Option<Revision> {
        self.stored.get(id).map(Setlist::rev)
    }
}

impl SetlistRepository for InMemorySetlistRepository {
    fn load(&self, id: &SetlistId) -> Option<Setlist> {
        self.stored.get(id).cloned()
    }

    fn list(&self) -> Vec<Setlist> {
        self.stored.values().cloned().collect()
    }

    fn save(&mut self, setlist: Setlist, expected: Option<Revision>) -> Result<(), SaveError> {
        let actual = self.stored_rev(setlist.id());
        if actual != expected {
            return Err(SaveError::Stale { expected, actual });
        }
        self.stored.insert(setlist.id().clone(), setlist);
        Ok(())
    }

    fn delete(&mut self, id: &SetlistId, expected: Revision) -> Result<(), SaveError> {
        let actual = self.stored_rev(id);
        if actual != Some(expected) {
            return Err(SaveError::Stale {
                expected: Some(expected),
                actual,
            });
        }
        self.stored.remove(id);
        Ok(())
    }
}
