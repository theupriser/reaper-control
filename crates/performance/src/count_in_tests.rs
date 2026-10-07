#![allow(clippy::unwrap_used, clippy::expect_used)]

use shared_kernel::{Bpm, Seconds};

use super::*;

fn s(v: f64) -> Seconds {
    Seconds::new(v).unwrap()
}

fn seg(start: f64, bpm: f64, beats: u32, unit: u32) -> TempoSegment {
    TempoSegment::new(
        s(start),
        Bpm::new(bpm).unwrap(),
        TimeSignature::new(beats, unit).unwrap(),
    )
}

fn map(segments: Vec<TempoSegment>) -> TempoMap {
    TempoMap::new(segments).unwrap()
}

struct Case {
    name: &'static str,
    map: TempoMap,
    cue: f64,
    tempo: Option<f64>,
    expect: f64,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "4/4 at 120 BPM",
            map: map(vec![seg(0.0, 120.0, 4, 4)]),
            cue: 30.0,
            tempo: None,
            expect: 4.0,
        },
        Case {
            name: "3/4 at 120 BPM",
            map: map(vec![seg(0.0, 120.0, 3, 4)]),
            cue: 30.0,
            tempo: None,
            expect: 3.0,
        },
        Case {
            name: "6/8 at 120 BPM",
            map: map(vec![seg(0.0, 120.0, 6, 8)]),
            cue: 30.0,
            tempo: None,
            expect: 3.0,
        },
        Case {
            name: "4/4 at 90 BPM",
            map: map(vec![seg(0.0, 90.0, 4, 4)]),
            cue: 30.0,
            tempo: None,
            expect: 16.0 / 3.0,
        },
        Case {
            name: "tempo change inside the count-in",
            map: map(vec![seg(0.0, 60.0, 4, 4), seg(28.0, 120.0, 4, 4)]),
            cue: 30.0,
            tempo: None,
            expect: 6.0,
        },
        Case {
            name: "!bpm overrides the map",
            map: map(vec![seg(0.0, 120.0, 4, 4)]),
            cue: 30.0,
            tempo: Some(60.0),
            expect: 8.0,
        },
        Case {
            name: "cue near 0 s is clamped",
            map: map(vec![seg(0.0, 120.0, 4, 4)]),
            cue: 1.5,
            tempo: None,
            expect: 1.5,
        },
        Case {
            name: "cue at 0 s",
            map: map(vec![seg(0.0, 120.0, 4, 4)]),
            cue: 0.0,
            tempo: None,
            expect: 0.0,
        },
    ]
}

#[test]
fn count_in_table() {
    let count_in = CountIn::default();
    println!("{:<36} {:>7} {:>8} {:>9}", "case", "cue", "!bpm", "lead-in");
    for case in cases() {
        let tempo = case.tempo.map(|b| Bpm::new(b).unwrap());
        let got = count_in.lead_in(&case.map, s(case.cue), tempo).get();
        println!(
            "{:<36} {:>6.2}s {:>8} {:>8.3}s",
            case.name,
            case.cue,
            case.tempo.map_or("-".to_string(), |b| b.to_string()),
            got
        );
        assert!(
            (got - case.expect).abs() < 1e-9,
            "{}: {got} != {}",
            case.name,
            case.expect
        );
    }
}

#[test]
fn signature_change_inside_the_count_in_is_walked_back() {
    // 120 BPM: last bar is 3/4 (1.5 s) from 28.5 s, the bar before is 4/4 (2 s).
    let m = map(vec![seg(0.0, 120.0, 4, 4), seg(28.5, 120.0, 3, 4)]);
    let got = CountIn::default().lead_in(&m, s(30.0), None).get();
    assert!((got - 3.5).abs() < 1e-9, "{got}");
}

#[test]
fn bpm_override_uses_the_signature_at_the_cue() {
    let m = map(vec![seg(0.0, 120.0, 4, 4), seg(10.0, 120.0, 3, 4)]);
    let got = CountIn::new(2).lead_in(&m, s(20.0), Some(Bpm::new(60.0).unwrap()));
    assert_eq!(got, s(6.0));
}

#[test]
fn more_bars_than_the_timeline_has_gives_the_whole_timeline() {
    let m = map(vec![seg(0.0, 120.0, 4, 4)]);
    assert_eq!(CountIn::new(100).lead_in(&m, s(7.0), None), s(7.0));
}

#[test]
fn invalid_maps_are_refused() {
    assert_eq!(TempoMap::new(vec![]), Err(InvalidTempoMap::Empty));
    assert_eq!(
        TempoMap::new(vec![seg(1.0, 120.0, 4, 4)]),
        Err(InvalidTempoMap::DoesNotCoverStart)
    );
    assert_eq!(
        TempoMap::new(vec![seg(0.0, 120.0, 4, 4), seg(0.0, 100.0, 4, 4)]),
        Err(InvalidTempoMap::NotAscending)
    );
    assert_eq!(
        TimeSignature::new(0, 4),
        Err(InvalidTempoMap::EmptySignature)
    );
}
