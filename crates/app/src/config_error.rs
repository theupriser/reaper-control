//! Why a config file could not be used.

/// A failure to read, understand or write the config file.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file could not be read or written.
    #[error("config file: {0}")]
    Io(#[from] std::io::Error),
    /// The file is not valid JSON of the expected shape.
    #[error("config file is not understood: {0}")]
    Parse(#[from] serde_json::Error),
    /// A value is outside its allowed range.
    #[error("config value out of range: {0}")]
    Invalid(String),
    /// The file was written by a newer version of the app.
    #[error("config file has schema version {found}, this app understands up to {supported}")]
    Newer {
        /// The version in the file.
        found: u32,
        /// The newest version this app reads.
        supported: u32,
    },
}
