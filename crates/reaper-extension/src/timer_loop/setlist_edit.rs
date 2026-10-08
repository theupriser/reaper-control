use protocol::EntryInfo;

/// A setlist the app wants stored, with the revision it was edited on.
pub(super) struct SetlistEdit<'a> {
    pub(super) id: &'a str,
    pub(super) name: &'a str,
    pub(super) entries: &'a [EntryInfo],
    pub(super) expected_revision: u64,
}
