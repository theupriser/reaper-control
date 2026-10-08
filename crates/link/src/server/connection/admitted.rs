/// A peer that passed the handshake.
pub(super) struct Admitted {
    /// The last event it saw, if it has seen any.
    pub(super) resume_from: Option<u64>,
}
