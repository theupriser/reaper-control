use std::collections::HashSet;

use shared_kernel::SongId;

use crate::{Edit, Entry, EntryId, InvalidSetlist, Rejection, Revision, SetlistEvent, SetlistId};

/// An ordered list of songs for one project. Editing is allowed with entries
/// whose song is gone; `dangling` reports them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setlist {
    id: SetlistId,
    name: String,
    entries: Vec<Entry>,
    next_entry: u64,
    rev: Revision,
}

fn clean_name(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

impl Setlist {
    /// An empty setlist at the initial revision.
    pub fn new(id: SetlistId, name: &str) -> Result<Self, InvalidSetlist> {
        Self::restore(id, name, Vec::new(), Revision::INITIAL)
    }

    /// A setlist as stored. Entry ids must be unique.
    pub fn restore(
        id: SetlistId,
        name: &str,
        entries: Vec<Entry>,
        rev: Revision,
    ) -> Result<Self, InvalidSetlist> {
        let name = clean_name(name).ok_or(InvalidSetlist::EmptyName)?;
        let mut seen = HashSet::new();
        if !entries.iter().all(|entry| seen.insert(entry.id())) {
            return Err(InvalidSetlist::DuplicateEntry);
        }
        let next_entry = match entries.iter().map(|entry| entry.id().get()).max() {
            Some(max) => max.checked_add(1).ok_or(InvalidSetlist::IdsExhausted)?,
            None => 0,
        };
        Ok(Self {
            id,
            name,
            entries,
            next_entry,
            rev,
        })
    }

    /// The setlist's identity.
    pub fn id(&self) -> &SetlistId {
        &self.id
    }

    /// The name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The entries, in order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The current revision.
    pub fn rev(&self) -> Revision {
        self.rev
    }

    /// The songs, in order.
    pub fn songs(&self) -> impl Iterator<Item = &SongId> {
        self.entries.iter().map(Entry::song)
    }

    /// Entries whose song the project no longer has.
    pub fn dangling(&self, is_known: impl Fn(&SongId) -> bool) -> Vec<EntryId> {
        let gone = |entry: &&Entry| !is_known(entry.song());
        self.entries.iter().filter(gone).map(Entry::id).collect()
    }

    /// Applies an edit made against `expected`. On success the revision rises by one.
    pub fn edit(&mut self, expected: Revision, edit: Edit) -> Result<SetlistEvent, Rejection> {
        if expected != self.rev {
            return Err(Rejection::Stale {
                expected,
                actual: self.rev,
            });
        }
        let event = match edit {
            Edit::Rename(name) => self.rename(&name)?,
            Edit::Add { song, at } => self.add(song, at)?,
            Edit::Remove(entry) => self.remove(entry)?,
            Edit::Move { entry, to } => self.move_entry(entry, to)?,
        };
        self.rev = self.rev.next();
        Ok(event)
    }

    fn index_of(&self, entry: EntryId) -> Result<usize, Rejection> {
        let found = self.entries.iter().position(|e| e.id() == entry);
        found.ok_or(Rejection::UnknownEntry)
    }

    fn rename(&mut self, name: &str) -> Result<SetlistEvent, Rejection> {
        let name = clean_name(name).ok_or(Rejection::EmptyName)?;
        if name == self.name {
            return Err(Rejection::NoChange);
        }
        self.name = name.clone();
        Ok(SetlistEvent::Renamed { name })
    }

    fn add(&mut self, song: SongId, at: Option<usize>) -> Result<SetlistEvent, Rejection> {
        let at = at.unwrap_or(self.entries.len());
        if at > self.entries.len() {
            return Err(Rejection::PositionOutOfRange);
        }
        let next = self
            .next_entry
            .checked_add(1)
            .ok_or(Rejection::IdsExhausted)?;
        let entry = EntryId::new(self.next_entry);
        self.next_entry = next;
        self.entries.insert(at, Entry::new(entry, song.clone()));
        Ok(SetlistEvent::EntryAdded { entry, song, at })
    }

    fn remove(&mut self, entry: EntryId) -> Result<SetlistEvent, Rejection> {
        let index = self.index_of(entry)?;
        self.entries.remove(index);
        Ok(SetlistEvent::EntryRemoved { entry })
    }

    fn move_entry(&mut self, entry: EntryId, to: usize) -> Result<SetlistEvent, Rejection> {
        let from = self.index_of(entry)?;
        if to >= self.entries.len() {
            return Err(Rejection::PositionOutOfRange);
        }
        if to == from {
            return Err(Rejection::NoChange);
        }
        let moved = self.entries.remove(from);
        self.entries.insert(to, moved);
        Ok(SetlistEvent::EntryMoved { entry, from, to })
    }
}
