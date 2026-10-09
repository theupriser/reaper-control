use performance::TempoMap;
use shared_kernel::Seconds;

use crate::{Marker, Region, Transport};

/// Everything the extension needs from REAPER. `ReaperRsAdapter` is the real
/// implementation, `FakeReaper` the deterministic one for tests and the simulator.
pub trait ReaperPort {
    /// The monotonic clock the performance is stepped with.
    fn now(&self) -> Seconds;
    /// Where the play position is.
    fn position(&self) -> Seconds;
    /// What the transport is doing.
    fn transport(&self) -> Transport;
    /// Whether REAPER's count-in is switched on.
    fn count_in(&self) -> bool;
    /// Counts the edits made to the project; it changes whenever regions, markers or tempo may have
    /// changed, so readers can skip re-reading while it stays the same. ExtState is not covered
    /// (REAPER does not count it); compare what `ext_state` returns.
    fn change_count(&self) -> u64;
    /// Names the project tab that is current; it differs between tabs and stays the same for a tab
    /// as long as it is open. Two projects can have the same change count, so this is how a switch
    /// between them is noticed.
    fn project_token(&self) -> u64;
    /// The regions of the project, in timeline order.
    fn regions(&self) -> Vec<Region>;
    /// The markers of the project, in timeline order.
    fn markers(&self) -> Vec<Marker>;
    /// The tempo and time signature changes.
    fn tempo_map(&self) -> TempoMap;
    /// A value in the project's ExtState.
    fn ext_state(&self, section: &str, key: &str) -> Option<String>;

    /// Starts or continues playback.
    fn play(&mut self);
    /// Pauses playback.
    fn pause(&mut self);
    /// Moves the play position, also while playing.
    fn seek(&mut self, position: Seconds);
    /// Switches REAPER's count-in.
    fn set_count_in(&mut self, enabled: bool);
    /// Stores a value in the project's ExtState.
    fn set_ext_state(&mut self, section: &str, key: &str, value: &str);
}
