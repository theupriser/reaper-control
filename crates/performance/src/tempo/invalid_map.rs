use thiserror::Error;

/// Why a tempo map or time signature could not be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidTempoMap {
    /// A time signature had zero beats per bar or a zero beat unit.
    #[error("time signature needs at least one beat and a beat unit")]
    EmptySignature,
    /// The map had no segments.
    #[error("tempo map has no segments")]
    Empty,
    /// Segments were not in strictly ascending order of start.
    #[error("tempo segments must start in ascending order")]
    NotAscending,
    /// The first segment started after the beginning of the timeline.
    #[error("first tempo segment must start at or before 0 s")]
    DoesNotCoverStart,
}
