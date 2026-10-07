//! Wire protocol between the REAPER extension and the app (SPEC §2.2).
//! The types here are the single source for the UI's TypeScript types (WP 0.4).

mod app_state;
mod command;
pub mod frame;
pub mod message;
mod phase;

#[cfg(test)]
mod generated;

pub use app_state::AppState;
pub use command::Command;
pub use phase::Phase;
