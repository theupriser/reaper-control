//! Asking REAPER to quit and starting it again, for the extension update.

mod control;
mod fake_control;
mod system_control;

pub use control::ReaperControl;
pub use fake_control::FakeReaperControl;
pub use system_control::SystemReaperControl;
