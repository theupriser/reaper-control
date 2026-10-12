//! Why the diagnostics bundle could not be made.

/// A failure to write the bundle.
#[derive(Debug, thiserror::Error)]
pub enum DiagnosticsError {
    /// A file could not be read or written.
    #[error("diagnostics bundle: {0}")]
    Io(#[from] std::io::Error),
    /// The zip file could not be written.
    #[error("diagnostics bundle: {0}")]
    Zip(#[from] zip::result::ZipError),
}
