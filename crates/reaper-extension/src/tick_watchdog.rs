mod verdict;

use std::time::Duration;

pub use verdict::Verdict;

/// A tick should cost well under a millisecond; 5 ms is already a lot of REAPER's main thread.
const BUDGET: Duration = Duration::from_millis(5);
/// A tick this slow is stalling REAPER's user interface.
const STALL: Duration = Duration::from_millis(50);
/// Stalls in a row before the extension gives up. One hiccup (a disk spike) is forgiven.
const STALLS_IN_A_ROW: u32 = 3;

/// Judges how long each tick took (R10): slow ticks are reported, repeated stalls disable the
/// extension so it can never keep REAPER's main thread hostage on stage.
#[derive(Debug, Default)]
pub struct TickWatchdog {
    stalls_in_a_row: u32,
    over_budget: u64,
    slowest: Duration,
}

impl TickWatchdog {
    /// A watchdog that has seen no ticks.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one tick and says what to do about it.
    pub fn judge(&mut self, elapsed: Duration) -> Verdict {
        self.slowest = self.slowest.max(elapsed);
        if elapsed >= STALL {
            self.stalls_in_a_row += 1;
        } else {
            self.stalls_in_a_row = 0;
        }
        if self.stalls_in_a_row >= STALLS_IN_A_ROW {
            Verdict::Disable
        } else if elapsed > BUDGET {
            self.over_budget += 1;
            Verdict::OverBudget
        } else {
            Verdict::Within
        }
    }

    /// Ticks that went over the budget so far.
    pub fn over_budget(&self) -> u64 {
        self.over_budget
    }

    /// The slowest tick so far.
    pub fn slowest(&self) -> Duration {
        self.slowest
    }
}

#[cfg(test)]
mod tests;
