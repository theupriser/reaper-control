#![allow(clippy::unwrap_used, clippy::expect_used)]

use performance::{TempoMap, TimeSignature};
use proptest::prelude::*;
use shared_kernel::{Bpm, Seconds};

use super::*;

#[derive(Debug, Clone)]
enum Op {
    Play,
    Pause,
    Seek(f64),
    Advance(f64),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        Just(Op::Play),
        Just(Op::Pause),
        (0.0..500.0f64).prop_map(Op::Seek),
        (-5.0..30.0f64).prop_map(Op::Advance),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn the_clock_never_runs_backwards_and_the_playhead_follows_it(
        ops in proptest::collection::vec(op(), 0..60),
    ) {
        let tempo = TempoMap::constant(Bpm::new(120.0).unwrap(), TimeSignature::common());
        let mut reaper = FakeReaper::new(Vec::new(), Vec::new(), tempo);
        for op in ops {
            let (now, position, transport) = (reaper.now().get(), reaper.position().get(), reaper.transport());
            match op {
                Op::Play => reaper.play(),
                Op::Pause => reaper.pause(),
                Op::Seek(to) => reaper.seek(Seconds::new(to).unwrap()),
                Op::Advance(by) => {
                    reaper.advance(Seconds::new(by).unwrap());
                    prop_assert!(reaper.now().get() >= now);
                    let moved = reaper.position().get() - position;
                    if transport == Transport::Playing && by > 0.0 {
                        prop_assert!((moved - by).abs() < 1e-9);
                    } else {
                        prop_assert!(moved == 0.0);
                    }
                }
            }
        }
    }

    #[test]
    fn a_scenario_always_gives_the_same_trace(
        end in 5.0..60.0f64,
        gap in 0.0..20.0f64,
        run_for in 0.0..120.0f64,
        hard_stop in any::<bool>(),
    ) {
        let json = format!(
            r#"{{"name":"p","songs":[
                {{"id":"A","start":0,"end":{end},"hard_stop":{hard_stop}}},
                {{"id":"B","start":{start},"end":{finish}}}],
              "steps":[{{"do":"play"}},{{"do":"advance","seconds":{run_for}}},{{"do":"play"}},{{"do":"advance","seconds":{run_for}}}]}}"#,
            start = end + gap,
            finish = end + gap + 10.0,
        );
        let scenario = Scenario::from_json(&json).unwrap();
        let first = ScenarioRunner::run(&scenario).unwrap();
        let second = ScenarioRunner::run(&scenario).unwrap();
        prop_assert_eq!(first, second);
    }
}
