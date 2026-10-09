#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use performance::{TempoMap, TimeSignature};
use shared_kernel::{Bpm, Seconds};

use super::*;

fn t(value: f64) -> Seconds {
    Seconds::new(value).unwrap()
}

fn fake() -> FakeReaper {
    let tempo = TempoMap::constant(Bpm::new(120.0).unwrap(), TimeSignature::common());
    FakeReaper::new(Vec::new(), Vec::new(), tempo)
}

#[test]
fn the_playhead_moves_only_while_playing() {
    let mut reaper = fake();
    reaper.advance(t(1.0));
    assert_eq!((reaper.now().get(), reaper.position().get()), (1.0, 0.0));
    reaper.play();
    reaper.advance(t(2.0));
    assert_eq!((reaper.now().get(), reaper.position().get()), (3.0, 2.0));
    reaper.pause();
    reaper.advance(t(5.0));
    assert_eq!(reaper.position().get(), 2.0);
    assert_eq!(reaper.transport(), Transport::Paused);
}

#[test]
fn seeking_works_in_every_transport_state() {
    let mut reaper = fake();
    reaper.seek(t(12.0));
    assert_eq!(reaper.position().get(), 12.0);
    reaper.play();
    reaper.seek(t(3.0));
    reaper.advance(t(1.0));
    assert_eq!(reaper.position().get(), 4.0);
}

#[test]
fn time_never_runs_backwards() {
    let mut reaper = fake();
    reaper.advance(t(2.0));
    reaper.advance(t(-1.0));
    reaper.advance(t(0.0));
    assert_eq!(reaper.now().get(), 2.0);
}

#[test]
fn pausing_a_stopped_transport_keeps_it_stopped() {
    let mut reaper = fake();
    reaper.pause();
    assert_eq!(reaper.transport(), Transport::Stopped);
}

#[test]
fn ext_state_and_count_in_are_remembered() {
    let mut reaper = fake();
    assert_eq!(reaper.ext_state("RC", "setlist"), None);
    reaper.set_ext_state("RC", "setlist", "one");
    reaper.set_ext_state("RC", "setlist", "two");
    assert_eq!(reaper.ext_state("RC", "setlist").as_deref(), Some("two"));
    assert_eq!(reaper.ext_state("Other", "setlist"), None);
    reaper.set_count_in(true);
    assert!(reaper.count_in());
}

fn scenario_files() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testing/scenarios");
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_scenario_file_passes() {
    let files = scenario_files();
    assert!(files.len() >= 15);
    for file in files {
        let scenario = Scenario::from_json(&std::fs::read_to_string(&file).unwrap()).unwrap();
        let trace =
            ScenarioRunner::run(&scenario).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        println!("== {} ({})\n{trace}", scenario.name, file.display());
    }
}

fn run(json: &str) -> Result<Trace, ScenarioError> {
    ScenarioRunner::run(&Scenario::from_json(json)?)
}

#[test]
fn a_wrong_expectation_fails_and_says_why() {
    let json = r#"{"name":"x","songs":[{"id":"A","start":0,"end":10}],
        "steps":[{"do":"play"},{"do":"expect","phase":"Paused"}]}"#;
    let error = run(json).unwrap_err();
    assert_eq!(
        error.to_string(),
        "step 2 failed: expected phase [Paused], got [Playing]"
    );
}

#[test]
fn expected_events_must_match_in_order() {
    let json = r#"{"name":"x","songs":[{"id":"A","start":0,"end":10}],
        "steps":[{"do":"play"},{"do":"expect","events":["HandOverStarted"]}]}"#;
    assert!(run(json).unwrap_err().to_string().contains("events"));
}

#[test]
fn impossible_scenarios_are_refused() {
    let backwards = r#"{"name":"x","songs":[{"id":"A","start":5,"end":5}],"steps":[]}"#;
    assert!(matches!(run(backwards), Err(ScenarioError::Invalid(_))));
    let flag =
        r#"{"name":"x","songs":[],"steps":[{"do":"set_flag","flag":"loud","enabled":true}]}"#;
    assert!(matches!(run(flag), Err(ScenarioError::Invalid(_))));
    let time = r#"{"name":"x","songs":[],"steps":[{"do":"advance","seconds":-1}]}"#;
    assert!(matches!(run(time), Err(ScenarioError::Invalid(_))));
    assert!(matches!(
        Scenario::from_json("{"),
        Err(ScenarioError::Parse(_))
    ));
}

#[test]
fn the_trace_lists_commands_and_the_ticks_that_did_something() {
    let json = r#"{"name":"x","songs":[{"id":"A","start":0,"end":10}],
        "steps":[{"do":"play"},{"do":"advance","seconds":10.1}]}"#;
    let trace = run(json).unwrap();
    let text = trace.to_string();
    assert!(text.contains("Play -> effects [SeekTo"), "{text}");
    assert!(text.contains("PerformanceFinished"), "{text}");
    assert!(trace.lines().len() < 5, "quiet ticks are left out: {text}");
}

#[test]
fn counting_in_holds_the_playhead_only_when_playback_starts_from_a_stop_or_pause() {
    let mut reaper = fake();
    reaper.set_count_in(true);
    reaper.seek(t(10.0));
    reaper.play();
    reaper.advance(t(3.0));
    assert_eq!(reaper.position().get(), 10.0);
    reaper.advance(t(2.0));
    assert_eq!(reaper.position().get(), 11.0);

    reaper.seek(t(30.0));
    reaper.advance(t(1.0));
    assert_eq!(reaper.position().get(), 31.0);

    reaper.pause();
    reaper.play();
    reaper.set_count_in(false);
    reaper.advance(t(1.0));
    assert_eq!(
        reaper.position().get(),
        31.0,
        "the count-in runs on after the flag is cleared"
    );
}

#[test]
fn pausing_ends_a_count_in() {
    let mut reaper = fake();
    reaper.set_count_in(true);
    reaper.play();
    reaper.advance(t(1.0));
    reaper.pause();
    reaper.play();
    reaper.advance(t(1.0));
    assert_eq!(reaper.position().get(), 0.0);
    reaper.pause();
    reaper.set_count_in(false);
    reaper.play();
    reaper.advance(t(1.0));
    assert_eq!(reaper.position().get(), 1.0);
}
