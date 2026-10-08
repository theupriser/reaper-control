use std::sync::{Arc, Mutex};
use std::time::Duration;

use protocol::Command;

use super::*;
use crate::fake_clock::FakeClock;
use crate::fake_driver::FakeDriver;

fn bus_with(driver: &Arc<FakeDriver>, events: Arc<EventBus>) -> (Arc<FakeClock>, CommandBus) {
    let clock = Arc::new(FakeClock::default());
    let bus = CommandBus::new(
        driver.clone(),
        events,
        clock.clone(),
        QueueSettings::default(),
    );
    (clock, bus)
}

fn recorded(events: &EventBus) -> Arc<Mutex<Vec<AppEvent>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    events.subscribe(move |event| {
        if let Ok(mut seen) = sink.lock() {
            seen.push(event.clone());
        }
    });
    seen
}

#[test]
fn a_dispatched_command_reaches_the_driver_and_is_announced() {
    let driver = Arc::new(FakeDriver::default());
    let events = Arc::new(EventBus::default());
    let seen = recorded(&events);
    let (_clock, bus) = bus_with(&driver, events);

    assert_eq!(bus.dispatch(Command::Play), Ok(()));
    assert_eq!(bus.dispatch(Command::Next), Ok(()));

    assert_eq!(driver.sent(), vec![Command::Play, Command::Next]);
    assert_eq!(
        seen.lock().map(|s| s.clone()).unwrap_or_default(),
        vec![
            AppEvent::CommandSent(Command::Play),
            AppEvent::CommandSent(Command::Next)
        ]
    );
}

#[test]
fn a_refused_command_is_announced_with_the_reason_and_not_sent() {
    let driver = Arc::new(FakeDriver::default());
    driver.set_disconnected(true);
    let events = Arc::new(EventBus::default());
    let seen = recorded(&events);
    let (_clock, bus) = bus_with(&driver, events);

    assert_eq!(
        bus.dispatch(Command::Play),
        Err(DispatchError::Driver(DriverError::NotConnected))
    );

    assert!(driver.sent().is_empty());
    assert_eq!(
        seen.lock().map(|s| s.clone()).unwrap_or_default(),
        vec![AppEvent::CommandRefused {
            command: Command::Play,
            error: DriverError::NotConnected
        }]
    );
}

#[test]
fn every_subscriber_hears_every_event_in_order() {
    let events = EventBus::default();
    let first = recorded(&events);
    let second = recorded(&events);
    events.publish(&AppEvent::CommandSent(Command::Play));
    events.publish(&AppEvent::CommandSent(Command::Pause));
    let expected = vec![
        AppEvent::CommandSent(Command::Play),
        AppEvent::CommandSent(Command::Pause),
    ];
    assert_eq!(
        first.lock().map(|s| s.clone()).unwrap_or_default(),
        expected
    );
    assert_eq!(
        second.lock().map(|s| s.clone()).unwrap_or_default(),
        expected
    );
}

fn heard(seen: &Arc<Mutex<Vec<AppEvent>>>) -> Vec<AppEvent> {
    seen.lock().map(|s| s.clone()).unwrap_or_default()
}

#[test]
fn a_rapid_repeat_is_dropped_and_announced_but_not_an_error() {
    let driver = Arc::new(FakeDriver::default());
    let events = Arc::new(EventBus::default());
    let seen = recorded(&events);
    let (clock, bus) = bus_with(&driver, events);

    assert_eq!(bus.dispatch(Command::Next), Ok(()));
    assert_eq!(bus.dispatch(Command::Next), Ok(()));
    assert_eq!(driver.sent(), vec![Command::Next]);
    assert_eq!(
        heard(&seen).last(),
        Some(&AppEvent::CommandDropped(Command::Next))
    );

    clock.advance(Duration::from_secs(1));
    assert_eq!(bus.dispatch(Command::Next), Ok(()));
    assert_eq!(driver.sent(), vec![Command::Next, Command::Next]);
}

#[test]
fn a_full_queue_refuses_until_the_extension_answers() {
    let driver = Arc::new(FakeDriver::default());
    let events = Arc::new(EventBus::default());
    let seen = recorded(&events);
    let clock = Arc::new(FakeClock::default());
    let settings = QueueSettings {
        timeout: Duration::from_secs(60),
        ..QueueSettings::default()
    };
    let bus = CommandBus::new(driver.clone(), events.clone(), clock.clone(), settings);
    for step in 0..32 {
        clock.advance(Duration::from_millis(300));
        let position = f64::from(step);
        assert_eq!(
            bus.dispatch(Command::Seek {
                position,
                count_in: false
            }),
            Ok(())
        );
    }
    clock.advance(Duration::from_millis(300));
    assert_eq!(bus.dispatch(Command::Play), Err(DispatchError::QueueFull));
    assert_eq!(
        heard(&seen).last(),
        Some(&AppEvent::CommandQueueFull(Command::Play))
    );

    events.publish(&AppEvent::CommandAcknowledged { id: 1 });
    assert_eq!(bus.dispatch(Command::Play), Ok(()));
}

#[test]
fn a_command_without_an_answer_times_out_once_and_an_answer_prevents_it() {
    let driver = Arc::new(FakeDriver::default());
    let events = Arc::new(EventBus::default());
    let seen = recorded(&events);
    let (clock, bus) = bus_with(&driver, events.clone());
    assert_eq!(bus.dispatch(Command::Play), Ok(()));
    assert_eq!(bus.dispatch(Command::Pause), Ok(()));
    events.publish(&AppEvent::CommandAcknowledged { id: 2 });

    clock.advance(Duration::from_secs(6));
    bus.expire();
    bus.expire();
    let timed_out: Vec<_> = heard(&seen)
        .into_iter()
        .filter(|event| matches!(event, AppEvent::CommandTimedOut { .. }))
        .collect();
    assert_eq!(
        timed_out,
        vec![AppEvent::CommandTimedOut {
            id: 1,
            command: Command::Play
        }]
    );
}

#[test]
fn losing_the_link_forgets_the_commands_in_flight() {
    let driver = Arc::new(FakeDriver::default());
    let events = Arc::new(EventBus::default());
    let seen = recorded(&events);
    let (clock, bus) = bus_with(&driver, events.clone());
    assert_eq!(bus.dispatch(Command::Play), Ok(()));
    events.publish(&AppEvent::LinkLost);
    clock.advance(Duration::from_secs(6));
    bus.expire();
    assert!(
        !heard(&seen)
            .iter()
            .any(|e| matches!(e, AppEvent::CommandTimedOut { .. }))
    );
}
