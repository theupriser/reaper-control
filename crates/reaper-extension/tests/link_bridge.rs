//! The bridge over a real loopback link and the fake REAPER: a client's Play reaches the
//! performance, and the new phase is pushed back.

use std::time::{Duration, Instant};

use link::{ClientConfig, Endpoint, LinkClient, LinkEvent};
use performance::{TempoMap, TimeSignature};
use protocol::{Command, Phase};
use reaper_extension::{LinkBridge, Log, TimerLoop};
use reaper_port::{FakeReaper, Region};
use shared_kernel::{Bpm, Seconds, SongId};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn a_command_over_the_link_moves_the_performance_and_the_state_comes_back() -> TestResult {
    let directory = std::env::temp_dir().join(format!("bridge-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let log = Log::new(directory.join("extension.log"));
    let song = Region {
        id: SongId::new("A"),
        name: "A".into(),
        start: Seconds::new(0.0)?,
        end: Seconds::new(10.0)?,
    };
    let reaper = FakeReaper::new(
        vec![song],
        vec![],
        TempoMap::constant(Bpm::FALLBACK, TimeSignature::common()),
    );
    let mut timer_loop = TimerLoop::new(reaper);
    let mut bridge = LinkBridge::start(&directory, &log)?;

    let endpoint = Endpoint::read(&directory.join("endpoint.json"))?;
    let (client, events) = LinkClient::start(ClientConfig::new(move || Some(endpoint.clone())));

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut sent = false;
    let mut playing = false;
    while Instant::now() < deadline && !playing {
        bridge.pump(&mut timer_loop, &log);
        timer_loop.tick();
        if !sent && client.is_connected() {
            client.send(Command::Play)?;
            sent = true;
        }
        while let Ok(event) = events.try_recv() {
            if matches!(event, LinkEvent::Live(live) if live.phase == Phase::Playing) {
                playing = true;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(playing, "no Playing live state arrived over the link");
    drop(bridge);
    assert!(!directory.join("endpoint.json").exists());
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}
