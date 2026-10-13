use std::path::Path;

/// Starts the app. The extension uses [`ProcessLauncher`](super::ProcessLauncher); tests use a
/// double that only records.
pub trait Launcher {
    /// Starts the application at `application` without waiting for it.
    fn launch(&mut self, application: &Path) -> Result<(), String>;
}
