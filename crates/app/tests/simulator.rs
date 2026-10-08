//! The simulator: the real performance rules over a fake REAPER, with time that only moves when told.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use app::app_event::AppEvent;
use app::command_bus::CommandBus;
use app::driver::Driver;
use app::event_bus::EventBus;
use app::fake_clock::FakeClock;
use app::fake_fault_check::FakeFaultCheck;
use app::fake_process_check::FakeProcessCheck;
use app::health_monitor::HealthMonitor;
use app::link_view_source::LinkViewSource;
use app::queue_settings::QueueSettings;
use app::simulator::{Simulator, sample_project};
use protocol::{Command, LinkStatus, Phase, WireEvent};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Run {
    simulator: Simulator,
    events: Arc<EventBus>,
    announced: Arc<Mutex<Vec<AppEvent>>>,
}

impl Run {
    fn start() -> Self {
        let events = Arc::new(EventBus::default());
        let announced = Arc::new(Mutex::new(Vec::new()));
        let recorder = Arc::clone(&announced);
        events.subscribe(move |event| {
            if let Ok(mut announced) = recorder.lock() {
                announced.push(event.clone());
            }
        });
        let health = Arc::new(HealthMonitor::new(
            events.clone(),
            Arc::new(FakeClock::default()),
            Arc::new(FakeProcessCheck::default()),
            Arc::new(FakeFaultCheck::default()),
        ));
        let simulator = Simulator::start(sample_project(), events.clone(), health, |_| {});
        Self {
            simulator,
            events,
            announced,
        }
    }

    fn performance_events(&self) -> Vec<WireEvent> {
        self.announced
            .lock()
            .map(|announced| {
                announced
                    .iter()
                    .filter_map(|event| match event {
                        AppEvent::PerformanceEvent(record) => Some(record.event.clone()),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn script(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.simulator.send(Command::Play)?;
        self.simulator.advance(Duration::from_secs(25));
        self.simulator.send(Command::Next)?;
        self.simulator.advance(Duration::from_secs(1));
        Ok(())
    }
}

#[test]
fn it_starts_connected_with_the_sample_songs_waiting() {
    let run = Run::start();
    let view = run.simulator.view();
    assert_eq!(
        view.status,
        LinkStatus::Connected {
            extension_version: "simulator".into()
        }
    );
    let names: Vec<_> = view
        .catalog
        .songs
        .iter()
        .map(|song| song.name.as_str())
        .collect();
    assert_eq!(names, ["Song A", "Song B"]);
    assert_eq!(view.live.map(|live| live.phase), Some(Phase::Idle));
}

#[test]
fn time_only_moves_when_told_and_the_end_of_a_song_hands_over() -> TestResult {
    let run = Run::start();
    run.simulator.send(Command::Play)?;
    run.simulator.advance(Duration::ZERO);
    let live = run.simulator.view().live.ok_or("no state")?;
    assert_eq!(live.phase, Phase::Playing);
    assert_eq!(live.position, 0.0);

    run.simulator.advance(Duration::from_secs(10));
    let live = run.simulator.view().live.ok_or("no state")?;
    assert!(
        (live.position - 10.0).abs() < 0.1,
        "position {}",
        live.position
    );
    assert_eq!(live.current_song, Some(0));

    run.simulator.advance(Duration::from_secs(11));
    let live = run.simulator.view().live.ok_or("no state")?;
    assert_eq!(live.current_song, Some(1));
    let events = run.performance_events();
    assert_eq!(events.first(), Some(&WireEvent::PerformanceStarted));
    assert!(matches!(
        events.get(1),
        Some(WireEvent::HandOverStarted { .. })
    ));
    assert!(matches!(
        events.get(2),
        Some(WireEvent::HandOverCompleted { .. })
    ));
    Ok(())
}

#[test]
fn next_on_the_last_song_is_refused_by_the_rules_and_acknowledged_as_the_extension_does()
-> TestResult {
    let run = Run::start();
    run.simulator.send(Command::Next)?;
    let id = run.simulator.send(Command::Next)?;
    run.simulator.advance(Duration::ZERO);
    assert!(
        run.performance_events()
            .iter()
            .any(|event| matches!(event, WireEvent::CommandRejected { .. }))
    );
    let announced = run.announced.lock().map_err(|_| "poisoned")?;
    assert!(announced.contains(&AppEvent::CommandAcknowledged { id }));
    Ok(())
}

#[test]
fn the_same_script_gives_the_same_events_and_state() -> TestResult {
    let first = Run::start();
    let second = Run::start();
    first.script()?;
    second.script()?;
    assert_eq!(first.performance_events(), second.performance_events());
    assert_eq!(first.simulator.view(), second.simulator.view());
    assert!(!first.performance_events().is_empty());
    Ok(())
}

#[test]
fn a_command_through_the_bus_does_not_wait_on_its_own_answer() -> TestResult {
    let run = Run::start();
    let simulator = Arc::new(run.simulator);
    let bus = CommandBus::new(
        simulator.clone(),
        run.events.clone(),
        Arc::new(FakeClock::default()),
        QueueSettings::default(),
    );
    bus.dispatch(Command::Play)?;
    simulator.advance(Duration::from_secs(1));
    let live = simulator.view().live.ok_or("no state")?;
    assert_eq!(live.phase, Phase::Playing);
    Ok(())
}
