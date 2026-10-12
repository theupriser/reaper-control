//! Asking whether REAPER is running.

mod check;
mod fake_check;
mod system_check;

pub use check::ProcessCheck;
pub use fake_check::FakeProcessCheck;
pub use system_check::SystemProcessCheck;
