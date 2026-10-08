use serde::{Deserialize, Serialize};

use crate::InvalidValue;

/// Tempo in beats per minute. Always finite and greater than zero.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Bpm(f64);

impl Bpm {
    /// The tempo to fall back on when nothing better is known.
    pub const FALLBACK: Self = Self(120.0);

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
