//! The companion extension loaded by REAPER (SPEC §2, S-9): the plugin entry, the panic guard,
//! the Faulted state, the safe-mode marker, the real `ReaperPort` and the timer loop that drives
//! the performance core. The link server follows in the next work package.
#![deny(unsafe_code)]

mod fault;
#[allow(unsafe_code)] // the one module that talks to REAPER and the C runtime (SPEC S-9.2)
mod foreign_interface;
mod log;
mod safe_mode_marker;
mod start_up;
mod tick_watchdog;
mod timer_loop;

pub use fault::Fault;
pub use log::Log;
pub use safe_mode_marker::SafeModeMarker;
pub use start_up::StartUp;
pub use tick_watchdog::{TickWatchdog, Verdict};
pub use timer_loop::TimerLoop;
