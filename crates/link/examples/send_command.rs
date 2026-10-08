//! Connects to a running extension and prints what it pushes. Each extra argument is a command
//! (`play`, `pause`, `next`, `previous`, `restart`), sent one second apart.
//! `cargo run -p link --example send_command -- <resource directory>/RC2/endpoint.json play next`

use std::time::{Duration, Instant};

use link::{ClientConfig, Endpoint, LinkClient, LinkEvent};
use protocol::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments
        .next()
        .ok_or("usage: send_command <endpoint.json> [commands]")?;
    let mut commands: Vec<Command> = Vec::new();
    for word in arguments {
        commands.push(match word.as_str() {
            "play" => Command::Play,
            "pause" => Command::Pause,
            "next" => Command::Next,
            "previous" => Command::Previous,
            "restart" => Command::RestartSong,
            other => return Err(format!("unknown command {other}").into()),
        });
    }
    let (client, events) = LinkClient::start(ClientConfig::new(move || {
        Endpoint::read(std::path::Path::new(&path)).ok()
    }));
    let start = Instant::now();
    let mut next_at = Duration::from_secs(1);
    let mut last_phase = None;
    while start.elapsed()
        < Duration::from_secs(1)
            + Duration::from_secs(commands.len() as u64)
            + Duration::from_secs(2)
    {
        if let Ok(event) = events.recv_timeout(Duration::from_millis(50)) {
            match event {
                LinkEvent::State(state) if last_phase != Some(state.phase) => {
                    last_phase = Some(state.phase);
                    println!(
                        "{:>5.1}s state: {:?} at {:.2} s",
                        start.elapsed().as_secs_f64(),
                        state.phase,
                        state.position
                    );
                }
                LinkEvent::State(_) => {}
                other => println!("{:>5.1}s {other:?}", start.elapsed().as_secs_f64()),
            }
        }
        if client.is_connected() && !commands.is_empty() && start.elapsed() >= next_at {
            let command = commands.remove(0);
            println!(
                "{:>5.1}s send {command:?}: {:?}",
                start.elapsed().as_secs_f64(),
                client.send(command)
            );
            next_at += Duration::from_secs(1);
        }
    }
    Ok(())
}
