//! The app's connection against a real server over loopback: connect, show state, send, lose it.

use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app::app_event::AppEvent;
use app::event_bus::EventBus;
use app::link_connection::LinkConnection;
use link::{CommandHandler, LinkServer, SendError};
use protocol::message::Outcome;
use protocol::{Command, LinkStatus, LinkView, Live, Phase};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Recorder(Mutex<Sender<Command>>);

impl CommandHandler for Recorder {
    fn handle(&self, command: Command) -> Outcome {
        if let Ok(sender) = self.0.lock() {
            let _ = sender.send(command);
        }
        Outcome::Done
    }
}

#[allow(clippy::panic)] // a view that never arrives fails the test
fn wait_for(connection: &LinkConnection, what: &str, done: impl Fn(&LinkView) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if done(&connection.view()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out: {what}; view {:?}", connection.view());
}

fn temporary_directory(name: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let directory = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    Ok(directory)
}

#[test]
fn a_missing_endpoint_file_shows_not_running_and_refuses_commands() -> TestResult {
    let directory = temporary_directory("app-link-missing")?;
    let connection = LinkConnection::start(directory.join("endpoint.json"), Arc::default(), |_| {});
    assert_eq!(connection.view(), LinkView::default());
    assert_eq!(connection.send(Command::Play), Err(SendError::NotConnected));
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

#[test]
fn it_follows_the_extension_and_sends_commands_to_it() -> TestResult {
    let directory = temporary_directory("app-link-live")?;
    let file = directory.join("endpoint.json");
    let (sender, received) = channel();
    let mut server = LinkServer::start("9.9.9", Recorder(Mutex::new(sender)))?;
    server.endpoint().write(&file)?;
    server.publish(Live {
        phase: Phase::Playing,
        position: 7.5,
        ..Live::default()
    });

    let changes = Arc::new(Mutex::new(0u32));
    let counted = Arc::clone(&changes);
    let connection = LinkConnection::start(file, Arc::default(), move |_| {
        if let Ok(mut count) = counted.lock() {
            *count += 1;
        }
    });
    wait_for(&connection, "connected with state", |view| {
        matches!(view.status, LinkStatus::Connected { .. })
            && view
                .live
                .as_ref()
                .is_some_and(|live| live.phase == Phase::Playing)
    });
    assert_eq!(
        connection.view().status,
        LinkStatus::Connected {
            extension_version: "9.9.9".into()
        }
    );
    assert!((connection.view().live.unwrap_or_default().position - 7.5).abs() < f64::EPSILON);
    assert!(changes.lock().map_err(|e| e.to_string())?.gt(&0));

    connection.send(Command::Next)?;
    assert_eq!(
        received.recv_timeout(Duration::from_secs(5))?,
        Command::Next
    );

    server.stop();
    wait_for(&connection, "not running after the stop", |view| {
        *view == LinkView::default()
    });
    assert_eq!(connection.send(Command::Play), Err(SendError::NotConnected));
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

struct Refuser;

impl CommandHandler for Refuser {
    fn handle(&self, _command: Command) -> Outcome {
        Outcome::Rejected {
            reason: "this is the last song".into(),
        }
    }
}

#[test]
fn connecting_and_a_refused_command_are_announced_on_the_event_bus() -> TestResult {
    let directory = temporary_directory("app-link-announce")?;
    let file = directory.join("endpoint.json");
    let mut server = LinkServer::start("9.9.9", Refuser)?;
    server.endpoint().write(&file)?;

    let events = Arc::new(EventBus::default());
    let (sender, announced) = channel();
    let sender = Mutex::new(sender);
    events.subscribe(move |event| {
        if let Ok(sender) = sender.lock() {
            let _ = sender.send(event.clone());
        }
    });
    let connection = LinkConnection::start(file, events, |_| {});
    assert_eq!(
        announced.recv_timeout(Duration::from_secs(5))?,
        AppEvent::LinkConnected {
            extension_version: "9.9.9".into()
        }
    );

    connection.send(Command::Next)?;
    let refused = announced.recv_timeout(Duration::from_secs(5))?;
    assert!(
        matches!(&refused, AppEvent::ExtensionRefused { reason, .. } if reason == "this is the last song"),
        "got {refused:?}"
    );

    server.stop();
    assert_eq!(
        announced.recv_timeout(Duration::from_secs(5))?,
        AppEvent::LinkLost
    );
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}
