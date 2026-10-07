use protocol::{EventRecord, WireEvent};

use super::*;

fn finished() -> WireEvent {
    WireEvent::PerformanceFinished
}

fn log_with(count: u64, capacity: usize) -> EventLog {
    let mut log = EventLog::new(capacity);
    for _ in 0..count {
        log.push(finished());
    }
    log
}

fn ids(replay: Replay) -> Option<Vec<u64>> {
    match replay {
        Replay::Events(records) => Some(records.iter().map(|r| r.id).collect()),
        Replay::Lost { .. } => None,
    }
}

#[test]
fn ids_start_at_one_and_rise_by_one() {
    let mut log = EventLog::new(10);
    assert_eq!(log.last_id(), 0);
    assert_eq!(log.push(finished()).id, 1);
    assert_eq!(log.push(finished()).id, 2);
    assert_eq!(log.last_id(), 2);
}

#[test]
fn a_new_app_gets_everything_and_a_caught_up_app_gets_nothing() {
    let log = log_with(3, 10);
    assert_eq!(ids(log.since(0)), Some(vec![1, 2, 3]));
    assert_eq!(ids(log.since(3)), Some(vec![]));
    assert_eq!(ids(EventLog::new(10).since(0)), Some(vec![]));
}

#[test]
fn a_reconnecting_app_gets_exactly_what_it_missed_in_order() {
    let log = log_with(7, 10);
    assert_eq!(ids(log.since(4)), Some(vec![5, 6, 7]));
}

#[test]
fn the_replay_carries_the_events_themselves() {
    let mut log = EventLog::new(10);
    log.push(finished());
    log.push(WireEvent::CommandRejected {
        reason: "no".into(),
    });
    assert_eq!(
        log.since(1),
        Replay::Events(vec![EventRecord {
            id: 2,
            event: WireEvent::CommandRejected {
                reason: "no".into()
            }
        }])
    );
}

#[test]
fn an_app_that_is_too_far_behind_is_told_so() {
    let log = log_with(10, 4);
    assert_eq!(ids(log.since(6)), Some(vec![7, 8, 9, 10]));
    assert_eq!(
        log.since(5),
        Replay::Lost {
            oldest_available: 7
        }
    );
    assert_eq!(
        log.since(0),
        Replay::Lost {
            oldest_available: 7
        }
    );
}

#[test]
fn an_app_ahead_of_a_restarted_extension_is_told_so() {
    let log = log_with(2, 10);
    assert_eq!(
        log.since(500),
        Replay::Lost {
            oldest_available: 1
        }
    );
}

#[test]
fn replay_is_exact_for_every_position() {
    for capacity in 1..8usize {
        for count in 0..20u64 {
            let log = log_with(count, capacity);
            for after in 0..=count + 2 {
                match log.since(after) {
                    Replay::Events(records) => {
                        let want: Vec<u64> = (after + 1..=count).collect();
                        let got: Vec<u64> = records.iter().map(|r| r.id).collect();
                        assert_eq!(got, want, "after {after} of {count} (cap {capacity})");
                    }
                    Replay::Lost { oldest_available } => {
                        let held = count.min(capacity as u64);
                        assert_eq!(oldest_available, count - held + 1);
                        assert!(after > count || after + 1 < oldest_available);
                    }
                }
            }
        }
    }
}
