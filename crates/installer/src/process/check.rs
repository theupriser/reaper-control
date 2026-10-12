//! The port through which the app asks whether REAPER is running.

/// Looks at the operating system's processes.
pub trait ProcessCheck: Send + Sync {
    /// Whether a REAPER process exists.
    fn reaper_is_running(&self) -> bool;
}
