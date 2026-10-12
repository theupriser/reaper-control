//! Tempo.

mod invalid_map;
mod map;
mod segment;
mod time_signature;

#[cfg(test)]
mod tests;

pub use invalid_map::InvalidTempoMap;
pub use map::TempoMap;
pub use segment::TempoSegment;
pub use time_signature::TimeSignature;
