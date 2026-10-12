//! The parts of the link the rest of the core refers to: why it dropped, how healthy it is, and
//! whether the show is running.

mod cause;
mod health;
mod performance_runs;

pub use cause::LinkCause;
pub use health::LinkHealth;
pub use performance_runs::performance_runs;
