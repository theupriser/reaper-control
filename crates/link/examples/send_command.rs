//! Connects to a running extension and prints what it pushes. Each extra argument is a command
//! (`play`, `pause`, `next`, `previous`, `restart`), sent one second apart.
//! `goto:<index>` jumps to the song at that position (from 0).
//! `active:<id>` (or `active:` for none) chooses the played setlist.
//! `save:<id>:<name>:<expected revision>:<song id>,<song id>` stores a setlist.
//! `cargo run -p link --example send_command -- <resource directory>/RC2/endpoint.json play next`

use std::time::{Duration, Instant};

use link::{ClientConfig, Endpoint, LinkClient, LinkEvent};
use protocol::{Command, EntryInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments
        .next()
        .ok_or("usage: send_command <endpoint.json> [commands]")?;
    let mut commands: Vec<Command> = Vec::new();
    for word in arguments {
        let command = match word.as_str() {
            "play" => Command::Play,
            "pause" => Command::Pause,
            "next" => Command::Next,
            "previous" => Command::Previous,
            "restart" => Command::RestartSong,
            "countin" => Command::ToggleCountInOnMarker,
            other if other.starts_with("goto:") => Command::GoToSong {
                index: other["goto:".len()..].parse()?,
            },
            other if other.starts_with("seekcue:") => Command::Seek {
                position: other["seekcue:".len()..].parse()?,
                count_in: true,
            },
            other => match other.strip_prefix("active:") {
                Some(id) => Command::SetActiveSetlist {
                    id: (!id.is_empty()).then(|| id.to_owned()),
                },
                None => match other.strip_prefix("save:") {
                    Some(specification) => save_command(specification)?,
                    None => return Err(format!("unknown command {other}").into()),
                },
            },
        };
        commands.push(command);
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
                LinkEvent::Live(state) if last_phase != Some(state.phase) => {
                    last_phase = Some(state.phase);
                    println!(
                        "{:>5.1}s live: {:?} ({:?}) at {:.2} s",
                        start.elapsed().as_secs_f64(),
                        state.phase,
                        state.transport,
                        state.position
                    );
                }
                LinkEvent::Live(live) => println!(
                    "{:>5.1}s   position {:.2} {:?}",
                    start.elapsed().as_secs_f64(),
                    live.position,
                    live.transport
                ),
                other => println!("{:>5.1}s {other:?}", start.elapsed().as_secs_f64()),
            }
        }
        if client.is_connected() && !commands.is_empty() && start.elapsed() >= next_at {
            let command = commands.remove(0);
            let shown = format!("{command:?}");
            println!(
                "{:>5.1}s send {shown}: {:?}",
                start.elapsed().as_secs_f64(),
                client.send(command)
            );
            next_at += Duration::from_secs(1);
        }
    }
    Ok(())
}

fn save_command(specification: &str) -> Result<Command, String> {
    let parts: Vec<&str> = specification.splitn(4, ':').collect();
    let [id, name, expected, songs] = parts.as_slice() else {
        return Err("save:<id>:<name>:<expected revision>:<song ids>".into());
    };
    let expected_revision = expected
        .parse()
        .map_err(|_| "expected revision is not a number")?;
    let entries = songs
        .split(',')
        .filter(|song| !song.is_empty())
        .zip(1..)
        .map(|(song_id, id)| EntryInfo {
            id,
            song_id: song_id.to_owned(),
        })
        .collect();
    Ok(Command::SaveSetlist {
        id: (*id).to_owned(),
        name: (*name).to_owned(),
        entries,
        expected_revision,
    })
}
