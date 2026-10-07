#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Property tests for the count-in maths: any valid tempo map, any cue.

use proptest::prelude::*;
use shared_kernel::{Bpm, Seconds};

use super::*;

fn signature() -> impl Strategy<Value = TimeSignature> {
    (1..=16_u32, prop::sample::select(vec![2_u32, 4, 8, 16]))
        .prop_map(|(beats, unit)| TimeSignature::new(beats, unit).unwrap())
}

fn bpm() -> impl Strategy<Value = Bpm> {
    (20.0..300.0_f64).prop_map(|value| Bpm::new(value).unwrap())
}

/// A map whose first segment starts at or before 0 and whose starts ascend strictly.
fn tempo_map() -> impl Strategy<Value = TempoMap> {
    (
        -5.0..=0.0_f64,
        prop::collection::vec((0.1..60.0_f64, bpm(), signature()), 0..6),
        bpm(),
        signature(),
    )
        .prop_map(|(first, rest, bpm, signature)| {
            let mut start = first;
            let mut segments = vec![TempoSegment::new(
                Seconds::new(start).unwrap(),
                bpm,
                signature,
            )];
            for (step, bpm, signature) in rest {
                start += step;
                segments.push(TempoSegment::new(
                    Seconds::new(start).unwrap(),
                    bpm,
                    signature,
                ));
            }
            TempoMap::new(segments).unwrap()
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    /// The lead-in is never negative, never longer than the cue, and never NaN.
    #[test]
    fn lead_in_stays_between_zero_and_the_cue(
        map in tempo_map(),
        cue in 0.0..600.0_f64,
        bars in 0..9_u32,
        tempo in prop::option::of(bpm()),
    ) {
        let lead_in = CountIn::new(bars).lead_in(&map, Seconds::new(cue).unwrap(), tempo).get();
        prop_assert!(lead_in.is_finite());
        prop_assert!(lead_in >= 0.0, "lead-in {lead_in}");
        prop_assert!(lead_in <= cue + 1e-9, "lead-in {lead_in} longer than cue {cue}");
    }

    /// More bars never give a shorter lead-in.
    #[test]
    fn more_bars_never_shorten_the_lead_in(
        map in tempo_map(),
        cue in 0.0..600.0_f64,
        bars in 0..8_u32,
        tempo in prop::option::of(bpm()),
    ) {
        let cue = Seconds::new(cue).unwrap();
        let fewer = CountIn::new(bars).lead_in(&map, cue, tempo).get();
        let more = CountIn::new(bars + 1).lead_in(&map, cue, tempo).get();
        prop_assert!(more + 1e-9 >= fewer, "{bars} bars {fewer}, {} bars {more}", bars + 1);
    }

    /// In a constant tempo the lead-in is exactly bars times the bar length, unless clamped.
    #[test]
    fn constant_tempo_is_bars_times_bar_length(
        tempo in bpm(),
        signature in signature(),
        cue in 0.0..600.0_f64,
        bars in 0..9_u32,
    ) {
        let map = TempoMap::constant(tempo, signature);
        let bar = signature.quarters_per_bar() * 60.0 / tempo.get();
        let lead_in = CountIn::new(bars).lead_in(&map, Seconds::new(cue).unwrap(), None).get();
        let expected = (f64::from(bars) * bar).min(cue);
        prop_assert!((lead_in - expected).abs() < 1e-6, "got {lead_in}, expected {expected}");
    }
}
