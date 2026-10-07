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
mod planned_song;
mod rejection;
mod song_window;

pub use effect::Effect;
pub use event::Event;
pub use flag::Flag;
pub use flags::Flags;
pub use hand_over_policy::HandOverPolicy;
pub use input::Input;
pub use output::Output;
pub use performance::Performance;
pub use phase::Phase;
pub use planned_song::PlannedSong;
pub use rejection::Rejection;
pub use song_window::SongWindow;

#[cfg(test)]
mod tests;
