//! The companion extension loaded by REAPER (SPEC §2, S-9). This is the skeleton (WP 3.1): the
//! plugin entry, the panic guard, the Faulted state and the safe-mode marker. The performance
//! core, the link server and the timer loop follow in the next work packages.
#![deny(unsafe_code)]

mod fault;
#[allow(unsafe_code)] // the one module that talks to REAPER and the C runtime (SPEC S-9.2)
mod foreign_interface;
mod log;
mod safe_mode_marker;
mod start_up;

pub use fault::Fault;
pub use log::Log;
pub use safe_mode_marker::SafeModeMarker;
pub use start_up::StartUp;
