//! The app's real link client against the real server (over a fake REAPER), with a proxy in
//! between that delays, reorders, cuts and corrupts the traffic.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app::AppEvent;
use app::ChaosProxy;
use app::ChaosSettings;
use app::EventBus;
use app::FakeClock;
use app::FakeExtension;
use app::FakeFaultCheck;
use app::FakeProcessCheck;
use app::HealthMonitor;
use app::LinkConnection;
use app::LinkHealth;
use app::LinkViewSource;
use app::sample_project;
use link::Endpoint;
use protocol::{Command, LinkStatus, LinkView, Phase};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const TICK: Duration = Duration::from_millis(20);

struct Rig {
    // Declared first so the client stops before the server and the proxy it talks to.
    connection: LinkConnection,
    health: Arc<HealthMonitor>,
    announced: Arc<Mutex<Vec<AppEvent>>>,
    views: Arc<Mutex<Vec<LinkView>>>,
    proxy: ChaosProxy,
    extension: FakeExtension,
    directory: PathBuf,
}

impl Rig {
    fn start(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let directory = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::create_dir_all(&directory)?;
        let extension = FakeExtension::start(sample_project(), TICK)?;
        let proxy = ChaosProxy::start(extension.address())?;
        Endpoint {
            port: proxy.address().port(),
            ..extension.endpoint().clone()
        }
        .write(&directory.join("endpoint.json"))?;

        let events = Arc::new(EventBus::default());
        let announced = Arc::new(Mutex::new(Vec::new()));
        let recorder = Arc::clone(&announced);
        events.subscribe(move |event| {
            if let Ok(mut announced) = recorder.lock() {
                announced.push(event.clone());
            }
        });
        let health = Arc::new(HealthMonitor::new(
            Arc::clone(&events),
            Arc::new(FakeClock::default()),
            Arc::new(FakeProcessCheck::default()),
            Arc::new(FakeFaultCheck::default()),
        ));
        let views = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&views);
        let connection = LinkConnection::start(
            directory.join("endpoint.json"),
            events,
            Arc::clone(&health),
            move |view| {
                if let Ok(mut recorded) = recorded.lock() {
                    recorded.push(view);
                }
            },
        );
        Ok(Self {
            connection,
            health,
            announced,
            views,
            proxy,
            extension,
            directory,
        })
    }

    #[allow(clippy::panic)] // a condition that never holds fails the test
    fn wait_for(&self, what: &str, done: impl Fn(&Self) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if done(self) {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "timed out: {what}; health {:?}, view {:?}",
            self.health.health(),
            self.connection.view()
        );
    }

    fn shows_songs(&self) -> bool {
        let view = self.connection.view();
        matches!(view.status, LinkStatus::Connected { .. }) && view.catalog.songs.len() == 2
    }

    fn connects_seen(&self) -> usize {
        self.announced
            .lock()
            .map(|announced| {
                announced
                    .iter()
                    .filter(|event| matches!(event, AppEvent::LinkConnected { .. }))
                    .count()
            })
            .unwrap_or(0)
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

fn chaos(settings: ChaosSettings) -> ChaosSettings {
    settings
}

#[test]
fn through_a_quiet_proxy_the_app_shows_the_project_and_plays_it() -> TestResult {
    let rig = Rig::start("chaos-quiet")?;
    rig.wait_for("the project", Rig::shows_songs);
    rig.connection.send(Command::Play)?;
    rig.wait_for("playing", |rig| {
        rig.connection
            .view()
            .live
            .is_some_and(|live| live.phase == Phase::Playing && live.position > 0.0)
    });
    assert_eq!(rig.health.health(), LinkHealth::Connected);
    assert_eq!(rig.extension.client_count(), 1);
    Ok(())
}

#[test]
fn a_link_that_is_cut_again_and_again_comes_back_each_time_and_stays_when_it_heals() -> TestResult {
    let rig = Rig::start("chaos-cut")?;
    rig.wait_for("the project", Rig::shows_songs);
    rig.proxy.set(chaos(ChaosSettings {
        cut_after_frames: Some(3),
        ..ChaosSettings::default()
    }));
    rig.wait_for("three more connects", |rig| rig.connects_seen() >= 4);
    rig.proxy.set(ChaosSettings::default());
    rig.wait_for("a link that stays", |rig| {
        let seen = rig.connects_seen();
        std::thread::sleep(Duration::from_millis(500));
        rig.shows_songs()
            && rig.connects_seen() == seen
            && rig.health.health() == LinkHealth::Connected
    });
    Ok(())
}

#[test]
fn a_slow_link_still_delivers_a_command_and_its_answer() -> TestResult {
    let rig = Rig::start("chaos-slow")?;
    rig.wait_for("the project", Rig::shows_songs);
    rig.proxy.set(ChaosSettings {
        delay: Duration::from_millis(150),
        ..ChaosSettings::default()
    });
    rig.connection.send(Command::Play)?;
    rig.wait_for("playing over the slow link", |rig| {
        rig.connection
            .view()
            .live
            .is_some_and(|live| live.phase == Phase::Playing)
    });
    Ok(())
}

#[test]
fn reordered_frames_never_move_the_shown_state_backwards() -> TestResult {
    let rig = Rig::start("chaos-reorder")?;
    rig.wait_for("the project", Rig::shows_songs);
    rig.connection.send(Command::Play)?;
    rig.proxy.set(ChaosSettings {
        swap_neighbours: true,
        ..ChaosSettings::default()
    });
    std::thread::sleep(Duration::from_secs(2));
    rig.proxy.set(ChaosSettings::default());
    let views = rig.views.lock().map_err(|_| "views lock")?;
    let sequences: Vec<u64> = views
        .iter()
        .filter_map(|view| view.live.as_ref().map(|live| live.sequence))
        .collect();
    assert!(sequences.len() > 10, "too few states: {}", sequences.len());
    assert!(
        sequences.windows(2).all(|pair| pair[0] <= pair[1]),
        "the shown state went back: {sequences:?}"
    );
    Ok(())
}

#[test]
fn garbage_from_the_server_side_drops_the_link_and_the_app_recovers() -> TestResult {
    let rig = Rig::start("chaos-garbage")?;
    rig.wait_for("the project", Rig::shows_songs);
    let connects = rig.connects_seen();
    rig.proxy.set(ChaosSettings {
        garbage_after_frames: Some(2),
        ..ChaosSettings::default()
    });
    rig.wait_for("a second connect", |rig| rig.connects_seen() > connects);
    rig.proxy.set(ChaosSettings::default());
    rig.wait_for("a healthy link", |rig| {
        rig.shows_songs() && rig.health.health() == LinkHealth::Connected
    });
    Ok(())
}
