//! The registry of client handles for the exit hook is shared by the whole process, so its test
//! has a file, and a process, of its own.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use link::{ClientConfig, CommandHandler, Endpoint, LinkClient, LinkEvent, LinkServer};
use protocol::Command;
use protocol::message::Outcome;

struct Handler;

impl CommandHandler for Handler {
    fn handle(&self, _: Command) -> Outcome {
        Outcome::Done
    }
}

fn handles() -> usize {
    link::EXIT_REGISTRY.snapshot().into_iter().flatten().count()
}

#[test]
fn the_handle_of_a_connected_client_is_kept_for_the_exit_hook_and_let_go_after()
-> Result<(), Box<dyn std::error::Error>> {
    let server = LinkServer::start("0.0.0-test", Handler)?;
    assert_eq!(handles(), 0);

    let endpoint = Arc::new(Mutex::new(Some(server.endpoint().clone())));
    let source: Arc<Mutex<Option<Endpoint>>> = Arc::clone(&endpoint);
    let (client, events) = LinkClient::start(ClientConfig::new(move || {
        source.lock().ok().and_then(|e| e.clone())
    }));
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if matches!(
            events.recv_timeout(Duration::from_millis(50)),
            Ok(LinkEvent::Connected { .. })
        ) {
            break;
        }
    }
    assert_eq!(handles(), 1);

    drop(client);
    let start = Instant::now();
    while server.client_count() > 0 && start.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(handles(), 0);
    Ok(())
}
