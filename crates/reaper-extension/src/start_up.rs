use crate::safe_mode_marker::SafeModeMarker;

/// What the extension does at load.
#[derive(Debug)]
pub enum StartUp {
    /// The last run ended cleanly. The marker is written and must be released at clean exit.
    Normal(SafeModeMarker),
    /// A marker from the last run was still there: REAPER did not shut down cleanly, so the
    /// extension stays disabled until the user re-enables it (SPEC S-9.4).
    SafeMode,
}
