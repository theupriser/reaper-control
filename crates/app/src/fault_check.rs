//! The port through which the app asks whether the extension has turned itself off.

/// Looks for the reason the extension wrote down when it disabled itself.
pub trait FaultCheck: Send + Sync {
    /// Why the extension is disabled, or `None` when it reported no fault.
    fn reason(&self) -> Option<String>;
}
