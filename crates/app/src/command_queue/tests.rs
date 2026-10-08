use std::time::Duration;

use super::*;
use crate::fake_clock::FakeClock;

fn queue(settings: QueueSettings) -> (Arc<FakeClock>, CommandQueue) {
    let clock = Arc::new(FakeClock::default());
    (clock.clone(), CommandQueue::new(clock, settings))
}

#[test]
fn an_identical_command_inside_the_window_is_a_repeat_until_the_window_passes() {
    let (clock, mut queue) = queue(QueueSettings::default());
    assert_eq!(queue.admit(&Command::Next), Ok(()));
    queue.track(1, Command::Next);
    clock.advance(Duration::from_millis(100));
    assert_eq!(queue.admit(&Command::Next), Err(QueueRejection::Repeat));
    assert_eq!(queue.admit(&Command::Previous), Ok(()));
    clock.advance(Duration::from_millis(200));
    assert_eq!(queue.admit(&Command::Next), Ok(()));
}

#[test]
fn seeks_to_different_positions_are_not_repeats() {
    let (_clock, mut queue) = queue(QueueSettings::default());
    let seek = |position| Command::Seek {
        position,
        count_in: false,
    };
    queue.track(1, seek(10.0));
    assert_eq!(queue.admit(&seek(20.0)), Ok(()));
    assert_eq!(queue.admit(&seek(10.0)), Err(QueueRejection::Repeat));
}

#[test]
fn a_full_queue_refuses_until_an_answer_makes_room() {
    let settings = QueueSettings {
        capacity: 2,
        ..QueueSettings::default()
    };
    let (_clock, mut queue) = queue(settings);
    queue.track(1, Command::Play);
    queue.track(2, Command::Pause);
    assert_eq!(queue.admit(&Command::Stop), Err(QueueRejection::Full));
    assert!(queue.acknowledge(1));
    assert_eq!(queue.admit(&Command::Stop), Ok(()));
    assert!(!queue.acknowledge(1));
}

#[test]
fn only_commands_past_the_timeout_expire_and_they_expire_once() {
    let (clock, mut queue) = queue(QueueSettings::default());
    queue.track(1, Command::Play);
    clock.advance(Duration::from_secs(3));
    queue.track(2, Command::Pause);
    clock.advance(Duration::from_secs(3));
    let expired = queue.expire();
    assert_eq!(expired.iter().map(|p| p.id).collect::<Vec<_>>(), vec![1]);
    assert!(queue.expire().is_empty());
    assert_eq!(queue.waiting(), 1);
}
