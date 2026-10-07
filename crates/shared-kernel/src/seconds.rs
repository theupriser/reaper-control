use serde::{Deserialize, Serialize};

use crate::InvalidValue;

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
