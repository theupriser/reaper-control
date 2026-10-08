//! The companion extension loaded by REAPER (SPEC §2, S-9): the plugin entry, the panic guard,
//! the Faulted state, the safe-mode marker, the real `ReaperPort` and the timer loop that drives
//! the performance core, and the bridge that joins it to the link server.
#![deny(unsafe_code)]

mod fault;
#[allow(unsafe_code)] // the one module that talks to REAPER and the C runtime (SPEC S-9.2)
mod foreign_interface;
mod link_bridge;
mod log;
mod safe_mode_marker;
mod start_up;
mod tick_watchdog;
mod timer_loop;

pub use fault::Fault;
pub use link_bridge::{BridgeError, LinkBridge};
pub use log::Log;
pub use safe_mode_marker::SafeModeMarker;
pub use start_up::StartUp;
pub use tick_watchdog::{TickWatchdog, Verdict};
pub use timer_loop::TimerLoop;
