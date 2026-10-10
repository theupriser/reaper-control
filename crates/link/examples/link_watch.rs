//! Keeps one client connected for a number of seconds and prints every change of connection,
//! catalog and phase, so a script can restart REAPER underneath it. It ends with the number of connections.
//! `cargo run -p link --example link_watch -- <resource directory>/RC2/endpoint.json <seconds>`

use std::time::{Duration, Instant};

use link::{ClientConfig, Endpoint, LinkClient, LinkEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments
        .next()
        .ok_or("usage: link_watch <endpoint.json> <seconds>")?;
    let seconds: u64 = arguments.next().map_or(Ok(60), |text| text.parse())?;
    let (_client, events) = LinkClient::start(ClientConfig::new(move || {
        Endpoint::read(std::path::Path::new(&path)).ok()
    }));
    let start = Instant::now();
    let mut connections = 0;
    let mut last_phase = None;
    while start.elapsed() < Duration::from_secs(seconds) {
        let Ok(event) = events.recv_timeout(Duration::from_millis(50)) else {
            continue;
        };
        let at = start.elapsed().as_secs_f64();
        match event {
            LinkEvent::Connected { .. } => {
                connections += 1;
                println!("{at:>6.1}s connected (connection {connections})");
            }
            LinkEvent::Live(live) if last_phase != Some(live.phase) => {
                last_phase = Some(live.phase);
                println!(
                    "{at:>6.1}s phase {:?}, transport {:?}, at {:.2} s",
                    live.phase, live.transport, live.position
                );
            }
            LinkEvent::Catalog(catalog) => {
                println!(
                    "{at:>6.1}s catalog revision {}, project {}, {} songs, {} cues",
                    catalog.revision,
                    catalog.project_id,
                    catalog.project_songs.len(),
                    catalog.cues.len()
                );
                for song in catalog.project_songs.iter().take(3) {
                    println!(
                        "         song {}: {} {:.2} to {:.2} s",
                        song.number, song.name, song.start, song.end
                    );
                }
            }
            LinkEvent::Live(_) => {}
            other => println!("{at:>6.1}s {other:?}"),
        }
    }
    println!("{connections} connections");
    Ok(())
}
