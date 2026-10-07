use thiserror::Error;

/// Why a value could not be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidValue {
    /// The number was NaN or infinite.
    #[error("value is not a finite number")]
    NotFinite,
    /// The number was below the allowed minimum.
    #[error("value is out of range")]
    OutOfRange,
}
