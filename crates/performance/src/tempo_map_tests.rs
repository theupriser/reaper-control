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

#[test]
fn bars_in_a_constant_tempo_are_whole_bar_lengths() {
    let m = map(vec![seg(0.0, 120.0, 4, 4)]);
    assert_eq!(m.bars_before(s(30.0), 2), s(4.0));
    assert_eq!(m.bars_before(s(30.0), 1), s(2.0));
}

#[test]
fn a_signature_change_inside_the_bars_is_walked_back() {
    // 120 BPM: the last bar is 3/4 (1.5 s) from 28.5 s, the bar before is 4/4 (2 s).
    let m = map(vec![seg(0.0, 120.0, 4, 4), seg(28.5, 120.0, 3, 4)]);
    let got = m.bars_before(s(30.0), 2).get();
    assert!((got - 3.5).abs() < 1e-9, "{got}");
}

#[test]
fn more_bars_than_the_timeline_has_gives_the_whole_timeline() {
    let m = map(vec![seg(0.0, 120.0, 4, 4)]);
    assert_eq!(m.bars_before(s(7.0), 100), s(7.0));
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
