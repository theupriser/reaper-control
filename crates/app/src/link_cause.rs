//! Why the extension cannot be reached.

/// The reason the link is dead, as far as the app can tell from the outside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkCause {
    /// REAPER is not running.
    ReaperNotRunning,
    /// REAPER is running but the extension does not answer: it is not loaded, or it has not
    /// started listening yet.
    ExtensionNotLoaded,
    /// The extension speaks another protocol than this app.
    ExtensionOutdated {
        /// The protocol version the extension announced; 0 when it announced none.
        found: u32,
    },
}
