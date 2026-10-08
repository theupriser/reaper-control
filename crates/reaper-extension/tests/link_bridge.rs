//! The bridge over a real loopback link and the fake REAPER: a client's Play reaches the
//! performance, and the new phase is pushed back.

use std::time::{Duration, Instant};

use link::{ClientConfig, Endpoint, LinkClient, LinkEvent};
use performance::{TempoMap, TimeSignature};
use protocol::{Command, EntryInfo, Phase, WireEvent};
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

#[test]
fn a_saved_setlist_comes_back_in_the_catalog_and_a_stale_save_is_reported() -> TestResult {
    let directory = std::env::temp_dir().join(format!("bridge-save-{}", std::process::id()));
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

    let save = |expected_revision| Command::SaveSetlist {
        id: "sat".into(),
        name: "Saturday".into(),
        entries: vec![EntryInfo {
            id: 1,
            song_id: "A".into(),
        }],
        expected_revision,
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut sent = false;
    let (mut saved, mut refused) = (false, false);
    while Instant::now() < deadline && !(saved && refused) {
        bridge.pump(&mut timer_loop, &log);
        timer_loop.tick();
        if !sent && client.is_connected() {
            client.send(save(0))?;
            client.send(save(0))?;
            sent = true;
        }
        while let Ok(event) = events.try_recv() {
            match event {
                LinkEvent::Catalog(catalog) if catalog.setlists.len() == 1 => saved = true,
                LinkEvent::Event(record)
                    if matches!(record.event, WireEvent::CommandRejected { .. }) =>
                {
                    refused = true;
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(saved, "the saved setlist never reached the client");
    assert!(refused, "the stale save was not reported");
    drop(bridge);
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}

#[test]
fn every_event_a_client_receives_is_also_in_the_journal() -> TestResult {
    let directory = std::env::temp_dir().join(format!("bridge-journal-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let log = Log::new(directory.join("extension.log"));
    let mut songs = Vec::new();
    for (name, start) in [("A", 0.0), ("B", 10.0)] {
        songs.push(Region {
            id: SongId::new(name),
            name: name.into(),
            start: Seconds::new(start)?,
            end: Seconds::new(start + 10.0)?,
        });
    }
    let reaper = FakeReaper::new(
        songs,
        vec![],
        TempoMap::constant(Bpm::FALLBACK, TimeSignature::common()),
    );
    let mut timer_loop = TimerLoop::new(reaper);
    let mut bridge = LinkBridge::start(&directory, &log)?;
    let endpoint = Endpoint::read(&directory.join("endpoint.json"))?;
    let (client, events) = LinkClient::start(ClientConfig::new(move || Some(endpoint.clone())));

    let commands = [Command::Play, Command::Next, Command::Next];
    let mut sent = 0;
    let mut received = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && received.len() < 3 {
        bridge.pump(&mut timer_loop, &log);
        timer_loop.tick();
        if client.is_connected()
            && received.len() == sent
            && let Some(command) = commands.get(sent)
        {
            client.send(command.clone())?;
            sent += 1;
        }
        while let Ok(event) = events.try_recv() {
            if let LinkEvent::Event(record) = event {
                received.push(record.event);
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(received.len(), 3, "events seen: {received:?}");

    let journal = std::fs::read_to_string(directory.join("journal.log"))?;
    let written: Vec<WireEvent> = journal
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .map(|value| Ok(serde_json::from_value(value?["event"].clone())?))
        .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    assert_eq!(written, received);
    drop(bridge);
    std::fs::remove_dir_all(&directory)?;
    Ok(())
}
