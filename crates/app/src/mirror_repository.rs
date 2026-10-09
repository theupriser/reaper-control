//! The port through which the app keeps the restore-only copy of the setlists.

use protocol::SetlistInfo;

use crate::mirror_error::MirrorError;

/// Where the copy of a project's setlists lives; files in production, memory in tests.
pub trait MirrorRepository: Send + Sync {
    /// Replaces the copy of this project's setlists.
    ///
    /// # Errors
    /// [`MirrorError`] for an unusable project id or a failed write.
    fn save(&self, project_id: &str, setlists: &[SetlistInfo]) -> Result<(), MirrorError>;

    /// The copy of this project's setlists; empty when there is none.
    ///
    /// # Errors
    /// [`MirrorError`] for an unusable project id or a damaged copy.
    fn restore(&self, project_id: &str) -> Result<Vec<SetlistInfo>, MirrorError>;
}
