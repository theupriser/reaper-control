//! Why an extension update did not happen.

use crate::InstallError;

/// Why an extension update did not happen. Nothing was changed unless the message says so.
#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("a performance is running; update when the show is over")]
    /// A song is playing, paused or handing over.
    PerformanceRunning,
    #[error("there is no update to install")]
    /// The installed extension is already the bundled one.
    NothingToUpdate,
    #[error("REAPER did not close; save or cancel in REAPER and try again ({0})")]
    /// REAPER refused or ignored the request to quit (an unsaved project, for instance).
    ReaperStayedOpen(String),
    #[error("the extension is updated, but REAPER did not start again: {0}")]
    /// The new extension is in place; REAPER has to be opened by hand.
    RestartFailed(String),
    #[error("{0}")]
    /// Putting the files in place failed; REAPER is closed.
    Install(#[from] InstallError),
}
