use std::time::Duration;

use super::{TickWatchdog, Verdict};

fn millis(value: u64) -> Duration {
    Duration::from_millis(value)
}

#[test]
fn a_quick_tick_is_within_budget() {
    let mut watchdog = TickWatchdog::new();
    assert_eq!(watchdog.judge(Duration::from_micros(200)), Verdict::Within);
    assert_eq!(watchdog.judge(millis(5)), Verdict::Within);
    assert_eq!(watchdog.over_budget(), 0);
}

#[test]
fn a_slow_tick_is_reported_and_counted() {
    let mut watchdog = TickWatchdog::new();
    assert_eq!(watchdog.judge(millis(6)), Verdict::OverBudget);
    assert_eq!(watchdog.judge(millis(49)), Verdict::OverBudget);
    assert_eq!(watchdog.over_budget(), 2);
    assert_eq!(watchdog.slowest(), millis(49));
}

#[test]
fn one_stall_is_forgiven_three_in_a_row_disable() {
    let mut watchdog = TickWatchdog::new();
    assert_eq!(watchdog.judge(millis(80)), Verdict::OverBudget);
    assert_eq!(watchdog.judge(millis(80)), Verdict::OverBudget);
    assert_eq!(watchdog.judge(millis(1)), Verdict::Within);
    assert_eq!(watchdog.judge(millis(80)), Verdict::OverBudget);
    assert_eq!(watchdog.judge(millis(80)), Verdict::OverBudget);
    assert_eq!(watchdog.judge(millis(80)), Verdict::Disable);
}
