//! Shared kernel: the few value types every context agrees on (SPEC §14.1).
//! Deliberately tiny. No I/O, no REAPER, no framework types.

use serde::{Deserialize, Serialize};
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

/// A point or span in time, in seconds. Always finite.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Seconds(f64);

impl Seconds {
    /// Creates a value; rejects NaN and infinities.
    pub fn new(value: f64) -> Result<Self, InvalidValue> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(InvalidValue::NotFinite)
        }
    }

    /// The raw number of seconds.
    pub fn get(self) -> f64 {
        self.0
    }
}

/// Tempo in beats per minute. Always finite and greater than zero.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Bpm(f64);

impl Bpm {
    /// Creates a value; rejects NaN, infinities, zero and negatives.
    pub fn new(value: f64) -> Result<Self, InvalidValue> {
        if !value.is_finite() {
            Err(InvalidValue::NotFinite)
        } else if value <= 0.0 {
            Err(InvalidValue::OutOfRange)
        } else {
            Ok(Self(value))
        }
    }

    /// The raw tempo.
    pub fn get(self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_rejects_non_finite() {
        assert_eq!(Seconds::new(f64::NAN), Err(InvalidValue::NotFinite));
        assert_eq!(Seconds::new(f64::INFINITY), Err(InvalidValue::NotFinite));
        assert_eq!(Seconds::new(12.5).map(Seconds::get), Ok(12.5));
    }

    #[test]
    fn bpm_must_be_positive_and_finite() {
        assert_eq!(Bpm::new(0.0), Err(InvalidValue::OutOfRange));
        assert_eq!(Bpm::new(-3.0), Err(InvalidValue::OutOfRange));
        assert_eq!(Bpm::new(f64::NAN), Err(InvalidValue::NotFinite));
        assert_eq!(Bpm::new(128.0).map(Bpm::get), Ok(128.0));
    }
}
