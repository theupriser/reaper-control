//! A mirror repository in memory.

use std::collections::BTreeMap;
use std::sync::Mutex;

use protocol::SetlistInfo;

use crate::mirror_error::MirrorError;
use crate::mirror_repository::MirrorRepository;

/// For tests: keeps the copies in memory, counts the writes, and can be told to refuse them.
#[derive(Debug, Default)]
pub struct FakeMirrorRepository {
    copies: Mutex<BTreeMap<String, Vec<SetlistInfo>>>,
    writes: Mutex<usize>,
    refuse_writes: Mutex<bool>,
}

impl FakeMirrorRepository {
    /// How many writes succeeded.
    #[must_use]
    pub fn writes(&self) -> usize {
        self.writes.lock().map(|writes| *writes).unwrap_or_default()
    }

    /// Makes every following `save` fail (or work again with `false`).
    pub fn refuse_writes(&self, refuse: bool) {
        if let Ok(mut flag) = self.refuse_writes.lock() {
            *flag = refuse;
        }
    }
}

impl MirrorRepository for FakeMirrorRepository {
    fn save(&self, project_id: &str, setlists: &[SetlistInfo]) -> Result<(), MirrorError> {
        if self.refuse_writes.lock().is_ok_and(|refuse| *refuse) {
            return Err(std::io::Error::other("the disk is full").into());
        }
        if let (Ok(mut copies), Ok(mut writes)) = (self.copies.lock(), self.writes.lock()) {
            copies.insert(project_id.to_owned(), setlists.to_vec());
            *writes += 1;
        }
        Ok(())
    }

    fn restore(&self, project_id: &str) -> Result<Vec<SetlistInfo>, MirrorError> {
        Ok(self
            .copies
            .lock()
            .ok()
            .and_then(|copies| copies.get(project_id).cloned())
            .unwrap_or_default())
    }
}
