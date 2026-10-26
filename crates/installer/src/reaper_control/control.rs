//! The port through which the app asks REAPER to quit and starts it again.

use crate::ReaperInstall;

/// Quits and starts REAPER. Quitting is a request: REAPER may still ask the person to save.
pub trait ReaperControl: Send + Sync {
    /// Asks REAPER to quit the normal way. Never kills it.
    fn quit(&self) -> Result<(), String>;

    /// Starts `install`'s REAPER.
    fn launch(&self, install: &ReaperInstall) -> Result<(), String>;
}
