//! Measures the link to a running extension: how often states arrive, whether any are missed, and
//! how long a command takes until its answer (WP 1.3, S3 numbers).
//! `cargo run --release -p link --example measure_link -- <resource directory>/RC2/endpoint.json [seconds] [commands] [play]`
//! With `play` the transport runs during the measurement, as on stage.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use link::{ClientConfig, Endpoint, LinkClient, LinkEvent};
use protocol::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments
        .next()
        .ok_or("usage: measure_link <endpoint.json> [seconds] [commands] [play]")?;
    let seconds: u64 = arguments.next().map_or(Ok(10), |text| text.parse())?;
    let commands: usize = arguments.next().map_or(Ok(100), |text| text.parse())?;
    let (client, events) = LinkClient::start(ClientConfig::new(move || {
        Endpoint::read(std::path::Path::new(&path)).ok()
    }));
    let started = Instant::now();
    while !client.is_connected() {
        if started.elapsed() > Duration::from_secs(10) {
            return Err("no connection within 10 s".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    if arguments.next().as_deref() == Some("play") {
        client.send(Command::Play)?;
        std::thread::sleep(Duration::from_secs(1));
    }
    let mut arrivals: Vec<Instant> = Vec::new();
    let (mut first_sequence, mut last_sequence, mut missed) = (None, 0_u64, 0_u64);
    let end = Instant::now() + Duration::from_secs(seconds);
    while Instant::now() < end {
        if let Ok(LinkEvent::Live(live)) = events.recv_timeout(Duration::from_millis(50)) {
            arrivals.push(Instant::now());
            first_sequence.get_or_insert(live.sequence);
            if last_sequence != 0 && live.sequence > last_sequence + 1 {
                missed += live.sequence - last_sequence - 1;
            }
            last_sequence = live.sequence;
        }
    }
    let intervals: Vec<f64> = arrivals
        .windows(2)
        .filter_map(|pair| Some(pair.get(1)?.duration_since(*pair.first()?).as_secs_f64() * 1000.0))
        .collect();
    println!(
        "states: {} in {seconds} s, missed by sequence: {missed}",
        arrivals.len()
    );
    summarise("interval between states", intervals);

    let mut sent: HashMap<u64, Instant> = HashMap::new();
    let mut round_trips: Vec<f64> = Vec::new();
    for _ in 0..commands {
        if let Ok(id) = client.send(Command::ToggleAutoResume) {
            sent.insert(id, Instant::now());
        }
        let wait_until = Instant::now() + Duration::from_millis(100);
        while Instant::now() < wait_until {
            if let Ok(LinkEvent::Ack { id, .. }) = events.recv_timeout(Duration::from_millis(5))
                && let Some(at) = sent.remove(&id)
            {
                round_trips.push(at.elapsed().as_secs_f64() * 1000.0);
            }
        }
    }
    println!(
        "commands: {} sent, {} answered",
        commands,
        round_trips.len()
    );
    summarise("round trip to the answer", round_trips);
    Ok(())
}

fn summarise(label: &str, mut values: Vec<f64>) {
    if values.is_empty() {
        println!("{label}: no data");
        return;
    }
    values.sort_by(f64::total_cmp);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let at = |fraction: f64| {
        values
            .get(((values.len() - 1) as f64 * fraction) as usize)
            .copied()
            .unwrap_or_default()
    };
    println!(
        "{label} (ms): mean {mean:.2}, median {:.2}, p99 {:.2}, max {:.2}, min {:.2}",
        at(0.5),
        at(0.99),
        at(1.0),
        at(0.0)
    );
}
