//! Where the app reads what the link last reported.

use protocol::LinkView;

/// The current view of REAPER: from the real link, or from the simulator.
pub trait LinkViewSource: Send + Sync {
    /// The current view.
    fn view(&self) -> LinkView;
}
