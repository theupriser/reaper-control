/// What a `LiveFeed` did with an update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applied {
    /// The update is newer than anything seen; show it.
    Accepted,
    /// The update is a duplicate or older than what is shown; drop it.
    Ignored,
}
