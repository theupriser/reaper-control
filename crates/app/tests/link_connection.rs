//! The app's connection against a real server over loopback: connect, show state, send, lose it.

use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use app::link_connection::LinkConnection;
use link::{CommandHandler, LinkServer, SendError};
use protocol::message::Outcome;
use protocol::{AppState, Command, LinkStatus, LinkView, Phase};

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
    let connection = LinkConnection::start(directory.join("endpoint.json"), |_| {});
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
    server.publish(AppState {
        phase: Phase::Playing,
        position: 7.5,
        ..AppState::default()
    });

    let changes = Arc::new(Mutex::new(0u32));
    let counted = Arc::clone(&changes);
    let connection = LinkConnection::start(file, move |_| {
        if let Ok(mut count) = counted.lock() {
            *count += 1;
        }
    });
    wait_for(&connection, "connected with state", |view| {
        matches!(view.status, LinkStatus::Connected { .. }) && view.state.phase == Phase::Playing
    });
    assert_eq!(
        connection.view().status,
        LinkStatus::Connected {
            extension_version: "9.9.9".into()
        }
    );
    assert!((connection.view().state.position - 7.5).abs() < f64::EPSILON);
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
