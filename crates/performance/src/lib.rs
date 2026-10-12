//! Performance context (core domain): the state machine that plays a setlist.
//! See SPEC §3 and §14.3. Pure: no I/O, no REAPER calls, no real clock.

mod effect;
mod event;
mod flag;
mod flags;
mod hand_over_policy;
mod input;
mod output;
mod performance;
mod phase;
mod reaper_stop_bridge;
mod rejection;
mod song;
mod tempo;

pub use effect::Effect;
pub use event::Event;
pub use flag::Flag;
pub use flags::Flags;
pub use hand_over_policy::HandOverPolicy;
pub use input::Input;
pub use output::Output;
pub use performance::Performance;
pub use phase::Phase;
pub use rejection::Rejection;
pub use song::PlannedSong;
pub use song::SongWindow;
pub use tempo::InvalidTempoMap;
pub use tempo::TempoMap;
pub use tempo::TempoSegment;
pub use tempo::TimeSignature;

#[cfg(test)]
mod performance_properties;
#[cfg(test)]
mod tests;
