//! Sends random transport commands to a running extension and checks, after each pause in the
//! commands, that the performance phase and REAPER's real transport agree.
//! `cargo run -p link --example random_actions -- <resource directory>/RC2/endpoint.json [steps] [seed]`

use std::time::{Duration, Instant};

use link::{ClientConfig, Endpoint, LinkClient, LinkEvent};
use protocol::{Command, Live, Phase, Transport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments
        .next()
        .ok_or("usage: random_actions <endpoint.json> [steps] [seed]")?;
    let steps: usize = arguments.next().map_or(Ok(60), |text| text.parse())?;
    let mut random: u64 = arguments.next().map_or(Ok(7), |text| text.parse())?;
    let mut next = move |limit: u64| {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        random % limit
    };

    let (client, events) = LinkClient::start(ClientConfig::new(move || {
        Endpoint::read(std::path::Path::new(&path)).ok()
    }));
    let mut latest: Option<Live> = None;
    let mut history: Vec<String> = Vec::new();
    let mut mismatches = 0;
    let mut checks = 0;
    let start = Instant::now();
    let wait = |latest: &mut Option<Live>, duration: Duration| {
        let until = Instant::now() + duration;
        while Instant::now() < until {
            if let Ok(LinkEvent::Live(live)) = events.recv_timeout(Duration::from_millis(20)) {
                *latest = Some(live);
            }
        }
    };

    wait(&mut latest, Duration::from_secs(2));
    for step in 0..steps {
        let command = match next(9) {
            0 | 1 => Command::Play,
            2 | 3 => Command::Pause,
            4 => Command::Next,
            5 => Command::Previous,
            6 => Command::RestartSong,
            7 => Command::GoToSong {
                index: next(10) as u32,
            },
            _ => Command::Seek {
                position: next(25) as f64,
                count_in: false,
            },
        };
        if next(4) == 0 {
            let _ = client.send(Command::ToggleAutoResume);
            history.push("ToggleAutoResume".into());
        }
        history.push(format!("{command:?}"));
        let _ = client.send(command);
        wait(&mut latest, Duration::from_millis(30 + next(300)));
        if step % 3 == 2 {
            wait(&mut latest, Duration::from_millis(900));
            if let Some(live) = &latest {
                checks += 1;
                let agrees = match live.phase {
                    Phase::Playing | Phase::CountingIn | Phase::HandingOver => {
                        live.transport == Transport::Playing
                    }
                    Phase::Paused => live.transport == Transport::Paused,
                    Phase::Idle | Phase::HardStopped | Phase::Finished => {
                        live.transport != Transport::Playing
                    }
                };
                if !agrees {
                    mismatches += 1;
                    println!(
                        "{:>6.1}s MISMATCH after step {step}: phase {:?}, transport {:?}, at {:.2} s; last commands: {}",
                        start.elapsed().as_secs_f64(),
                        live.phase,
                        live.transport,
                        live.position,
                        history
                            .iter()
                            .rev()
                            .take(6)
                            .rev()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
            }
        }
    }
    println!("{steps} steps, {checks} checks, {mismatches} mismatches");
    if checks == 0 {
        return Err(
            "never saw REAPER's state: is the endpoint path right and REAPER running?".into(),
        );
    }
    if mismatches > 0 {
        return Err("the phase and the transport disagreed".into());
    }
    Ok(())
}
