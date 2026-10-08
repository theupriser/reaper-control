use super::*;
use crate::fake_clock::FakeClock;
use crate::fake_driver::FakeDriver;

fn metered() -> (Arc<FakeClock>, Arc<EventBus>, MeteredDriver) {
    let clock = Arc::new(FakeClock::default());
    let events = Arc::new(EventBus::default());
    let driver = MeteredDriver::new(Arc::new(FakeDriver::default()), clock.clone(), &events);
    (clock, events, driver)
}

#[test]
fn the_delay_between_send_and_answer_is_recorded() {
    let (clock, events, driver) = metered();
    let first = driver.send(Command::Play);
    clock.advance(Duration::from_millis(30));
    let second = driver.send(Command::Stop);
    clock.advance(Duration::from_millis(10));
    assert_eq!(first, Ok(1));
    assert_eq!(second, Ok(2));

    events.publish(&AppEvent::CommandAcknowledged { id: 2 });
    events.publish(&AppEvent::CommandAcknowledged { id: 1 });

    assert_eq!(
        driver.stats(),
        LatencyStats {
            answered: 2,
            last: Duration::from_millis(40),
            slowest: Duration::from_millis(40),
        }
    );
}

#[test]
fn a_refused_command_counts_and_unknown_answers_are_ignored() {
    let (clock, events, driver) = metered();
    assert_eq!(driver.send(Command::Play), Ok(1));
    clock.advance(Duration::from_millis(5));
    events.publish(&AppEvent::CommandAcknowledged { id: 99 });
    events.publish(&AppEvent::ExtensionRefused {
        id: 1,
        reason: "no".into(),
    });
    assert_eq!(driver.stats().answered, 1);
    assert_eq!(driver.stats().last, Duration::from_millis(5));
}

#[test]
fn a_command_that_was_not_sent_is_not_waited_for() {
    let clock = Arc::new(FakeClock::default());
    let events = Arc::new(EventBus::default());
    let inner = Arc::new(FakeDriver::default());
    inner.set_disconnected(true);
    let driver = MeteredDriver::new(inner, clock, &events);
    assert_eq!(driver.send(Command::Play), Err(DriverError::NotConnected));
    events.publish(&AppEvent::CommandAcknowledged { id: 1 });
    assert_eq!(driver.stats(), LatencyStats::default());
}
