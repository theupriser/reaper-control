//! The system-specific step between copying the extension and REAPER loading it.

use std::path::Path;

/// Makes a copied extension loadable. On macOS that is signing and clearing quarantine; on
/// Windows nothing is needed (ADR-006).
pub trait ExtensionPreparer: Send + Sync {
    /// Prepares the file at `library`; the error says what failed.
    fn prepare(&self, library: &Path) -> Result<(), String>;
}
