//! Catalogue context: songs, cues and directives as seen in the REAPER project. See SPEC §4 and §14.3.
//! Pure domain code: no I/O, no REAPER calls.

mod cue;
mod directive;
mod directives;
mod invalid_song;
mod marker_name;
mod song;

pub use cue::Cue;
pub use directive::Directive;
pub use directives::Directives;
pub use invalid_song::InvalidSong;
pub use marker_name::MarkerName;
pub use song::Song;

#[cfg(test)]
mod tests;
