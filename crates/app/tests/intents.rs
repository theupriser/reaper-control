//! Intents from any controller take one path: judged against the state, then the command bus.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use app::AppEvent;
use app::CommandBus;
use app::EventBus;
use app::FakeClock;
use app::FakeDriver;
use app::Intent;
use app::IntentDispatcher;
use app::IntentError;
use app::IntentRefusal;
use app::QueueSettings;
use protocol::{Command, LinkView, Live, Phase};

#[test]
fn an_intent_past_the_last_song_is_announced_and_never_sent() {
    let driver = Arc::new(FakeDriver::default());
    let events = Arc::new(EventBus::default());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let recorder = Arc::clone(&seen);
    events.subscribe(move |event| {
        if let Ok(mut seen) = recorder.lock() {
            seen.push(event.clone());
        }
    });
    let bus = Arc::new(CommandBus::new(
        driver.clone(),
        events.clone(),
        Arc::new(FakeClock::default()),
        QueueSettings {
            repeat_window: Duration::ZERO,
            ..QueueSettings::default()
        },
    ));
    let intents = IntentDispatcher::new(bus, events, || LinkView {
        live: Some(Live {
            phase: Phase::Playing,
            setlist_id: Some("set".into()),
            current_song: Some(2),
            next_song: None,
            ..Live::default()
        }),
        ..LinkView::default()
    });
    assert_eq!(
        intents.dispatch(Intent::Next),
        Err(IntentError::Refused(IntentRefusal::NoNextSong))
    );
    assert_eq!(intents.dispatch(Intent::Previous), Ok(()));
    assert_eq!(driver.sent(), vec![Command::Previous]);
    assert_eq!(
        seen.lock()
            .map(|seen| seen.clone())
            .unwrap_or_default()
            .first(),
        Some(&AppEvent::IntentRefused {
            intent: Intent::Next,
            refusal: IntentRefusal::NoNextSong
        })
    );
}
