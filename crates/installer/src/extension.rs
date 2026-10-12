//! Extension.

mod bundled;
mod installer;
mod mac_preparer;
mod plain_preparer;
mod preparer;

pub use bundled::bundled_extension;
pub use installer::ExtensionInstaller;
pub use mac_preparer::MacExtensionPreparer;
pub use plain_preparer::PlainExtensionPreparer;
pub use preparer::ExtensionPreparer;
