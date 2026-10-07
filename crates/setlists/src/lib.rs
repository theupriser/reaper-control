//! Setlists context: creating, editing and validating setlists. See SPEC §14.3.
//! Pure domain code: no I/O, no REAPER calls.

mod edit;
mod entry;
mod entry_id;
mod in_memory_setlist_repository;
mod invalid_setlist;
mod rejection;
mod revision;
mod save_error;
mod setlist;
mod setlist_event;
mod setlist_id;
mod setlist_repository;

pub use edit::Edit;
pub use entry::Entry;
pub use entry_id::EntryId;
pub use in_memory_setlist_repository::InMemorySetlistRepository;
pub use invalid_setlist::InvalidSetlist;
pub use rejection::Rejection;
pub use revision::Revision;
pub use save_error::SaveError;
pub use setlist::Setlist;
pub use setlist_event::SetlistEvent;
pub use setlist_id::SetlistId;
pub use setlist_repository::SetlistRepository;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod setlist_properties;
