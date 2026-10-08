use std::sync::{Arc, Mutex};

use protocol::Command;

use super::*;
use crate::fake_driver::FakeDriver;

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
    let bus = CommandBus::new(driver.clone(), events);

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
    let bus = CommandBus::new(driver.clone(), events);

    assert_eq!(bus.dispatch(Command::Play), Err(DriverError::NotConnected));

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
