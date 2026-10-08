/// What the watchdog makes of one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The tick fit the budget.
    Within,
    /// The tick was slow but the extension carries on. Worth a log line.
    OverBudget,
    /// Ticks keep stalling REAPER; the extension must switch itself off.
    Disable,
}
