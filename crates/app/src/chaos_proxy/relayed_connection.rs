use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::frame_relay::FrameRelay;
use crate::chaos_settings::ChaosSettings;

/// How long a relay waits for bytes before it looks at the stop request.
const READ_POLL: Duration = Duration::from_millis(50);

/// One app connection and its server connection. Server to app goes frame by frame through the
/// chaos; app to server is passed on untouched.
pub(super) struct RelayedConnection;

impl RelayedConnection {
    pub(super) fn start(
        client: TcpStream,
        server: TcpStream,
        settings: &Arc<Mutex<ChaosSettings>>,
        stop: &Arc<AtomicBool>,
    ) {
        let (Ok(client_copy), Ok(server_copy)) = (client.try_clone(), server.try_clone()) else {
            return;
        };
        for stream in [&client, &server] {
            let _ = stream.set_read_timeout(Some(READ_POLL));
        }
        let settings = Arc::clone(settings);
        let down_stop = Arc::clone(stop);
        let up_stop = Arc::clone(stop);
        let _ = thread::Builder::new()
            .name("chaos-down".into())
            .spawn(move || {
                FrameRelay::new(settings).run(&server, &client, &down_stop);
                close(&server, &client);
            });
        let _ = thread::Builder::new()
            .name("chaos-up".into())
            .spawn(move || {
                copy(&client_copy, &server_copy, &up_stop);
                close(&client_copy, &server_copy);
            });
    }
}

fn copy(mut from: &TcpStream, mut to: &TcpStream, stop: &AtomicBool) {
    let mut chunk = [0u8; 4096];
    while !stop.load(Ordering::SeqCst) {
        match from.read(&mut chunk) {
            Ok(0) => return,
            Ok(count) => {
                if to
                    .write_all(chunk.get(..count).unwrap_or_default())
                    .is_err()
                {
                    return;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(_) => return,
        }
    }
}

fn close(first: &TcpStream, second: &TcpStream) {
    let _ = first.shutdown(Shutdown::Both);
    let _ = second.shutdown(Shutdown::Both);
}
