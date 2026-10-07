use crate::InvalidTempoMap;

/// Beats per bar over a beat unit, for example 6/8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    beats: u32,
    unit: u32,
}

impl TimeSignature {
    /// Creates a signature; both numbers must be above zero.
    pub fn new(beats: u32, unit: u32) -> Result<Self, InvalidTempoMap> {
        if beats == 0 || unit == 0 {
            Err(InvalidTempoMap::EmptySignature)
        } else {
            Ok(Self { beats, unit })
        }
    }

    /// The common 4/4.
    pub fn common() -> Self {
        Self { beats: 4, unit: 4 }
    }

    /// Length of one bar in quarter notes (6/8 is 3, 3/4 is 3).
    pub fn quarters_per_bar(self) -> f64 {
        f64::from(self.beats) * 4.0 / f64::from(self.unit)
    }
}
