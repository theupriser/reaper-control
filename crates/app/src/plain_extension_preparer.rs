//! Leaves the copied extension as it is (Windows, and tests).

use std::path::Path;

use crate::extension_preparer::ExtensionPreparer;

/// Does nothing to the copied file.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlainExtensionPreparer;

impl ExtensionPreparer for PlainExtensionPreparer {
    fn prepare(&self, _library: &Path) -> Result<(), String> {
        Ok(())
    }
}
