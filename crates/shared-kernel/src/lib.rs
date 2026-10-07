//! Shared kernel: the few value types every context agrees on (SPEC §14.1).
//! Deliberately tiny. No I/O, no REAPER, no framework types.

mod bpm;
mod invalid_value;
mod seconds;
mod song_id;

pub use bpm::Bpm;
pub use invalid_value::InvalidValue;
pub use seconds::Seconds;
pub use song_id::SongId;

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
