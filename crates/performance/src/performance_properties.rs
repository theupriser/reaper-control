#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Property tests: random setlists and random input sequences must never break
//! the rules of the state machine (SPEC §3).

use proptest::prelude::*;
use shared_kernel::Seconds;

use super::*;

fn t(value: f64) -> Seconds {
    Seconds::new(value).unwrap()
}

/// Up to six songs in ascending order; a gap of 0 makes two songs contiguous.
fn setlist(min: usize, hard_stops: bool) -> impl Strategy<Value = Vec<PlannedSong>> {
    prop::collection::vec((0.0..8.0_f64, 0.5..20.0_f64, any::<bool>()), min..6).prop_map(
        move |specs| {
            let mut cursor = 0.0;
            let mut songs = Vec::new();
            for (index, (gap, length, hard_stop)) in specs.into_iter().enumerate() {
                let gap = if gap < 2.0 { 0.0 } else { gap };
                let start = cursor + gap;
                cursor = start + length;
                songs.push(PlannedSong {
                    song_id: format!("song-{index}"),
                    window: SongWindow::new(t(start), t(cursor)).unwrap(),
                    hard_stop: hard_stops && hard_stop,
                    hard_stop_marker: None,
                });
            }
            songs
        },
    )
}

#[derive(Debug, Clone)]
enum Op {
    Tick { dt: f64, position: f64 },
    Play,
    Pause,
    Next,
    Previous,
    RestartSong,
    Seek(f64),
    SeekCue { position: f64 },
    SetFlag(bool, bool),
}

fn op() -> impl Strategy<Value = Op> {
    let position = 0.0..140.0_f64;
    prop_oneof![
        8 => (0.0..2.5_f64, position.clone()).prop_map(|(dt, position)| Op::Tick { dt, position }),
        2 => Just(Op::Play),
        1 => Just(Op::Pause),
        1 => Just(Op::Next),
        1 => Just(Op::Previous),
        1 => Just(Op::RestartSong),
        1 => position.clone().prop_map(Op::Seek),
        1 => position.prop_map(|position| Op::SeekCue { position }),
        1 => (any::<bool>(), any::<bool>()).prop_map(|(a, b)| Op::SetFlag(a, b)),
    ]
}

fn flags() -> impl Strategy<Value = Flags> {
    (any::<bool>(), any::<bool>()).prop_map(|(autoplay, count_in)| Flags { autoplay, count_in })
}

fn input(op: &Op, now: &mut f64) -> Input {
    match *op {
        Op::Tick { dt, position } => {
            *now += dt;
            Input::Tick {
                now: t(*now),
                position: t(position),
            }
        }
        Op::Play => Input::Play,
        Op::Pause => Input::Pause,
        Op::Next => Input::Next,
        Op::Previous => Input::Previous,
        Op::RestartSong => Input::RestartSong,
        Op::Seek(position) => Input::Seek {
            position: t(position),
        },
        Op::SeekCue { position } => Input::SeekCue {
            position: t(position),
        },
        Op::SetFlag(autoplay, enabled) => Input::SetFlag {
            flag: if autoplay {
                Flag::Autoplay
            } else {
                Flag::CountIn
            },
            enabled,
        },
    }
}

fn is_running(phase: Phase) -> bool {
    matches!(
        phase,
        Phase::Playing | Phase::CountingIn | Phase::HandingOver
    )
}

fn only_rejection(out: &Output) -> bool {
    out.effects.is_empty()
        && !out.events.is_empty()
        && out
            .events
            .iter()
            .all(|e| matches!(e, Event::CommandRejected(_)))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Whatever happens, the invariants hold after every single step.
    #[test]
    fn invariants_hold_after_every_step(
        songs in setlist(0, true),
        flags in flags(),
        ops in prop::collection::vec(op(), 0..120),
    ) {
        let count = songs.len();
        let mut p = Performance::new(songs, flags, HandOverPolicy::default());
        let mut now = 0.0;
        let mut last_hand_over: Option<f64> = None;

        for op in &ops {
            let input = input(op, &mut now);
            let before = (p.phase(), p.current_index());
            let out = p.step(input);

            // The index is in range, and None exactly when there are no songs.
            match p.current_index() {
                Some(i) => prop_assert!(i < count),
                None => prop_assert_eq!(count, 0),
            }

            // The last Play or Pause effect decides the phase family.
            let last = out.effects.iter().rev().find_map(|e| match e {
                Effect::Play => Some(true),
                Effect::Pause => Some(false),
                _ => None,
            });
            if let Some(playing) = last {
                prop_assert_eq!(is_running(p.phase()), playing, "{:?} after {:?}", out.effects, input);
            }

            // A count-in is only switched on while counting in; counting in
            // needs the flag unless a seek already moved on.
            if out.effects.contains(&Effect::SetCountIn(true)) {
                prop_assert_eq!(p.phase(), Phase::CountingIn);
                prop_assert!(p.flags().count_in);
            }

            // Every position REAPER is asked to go to is on the timeline.
            for effect in &out.effects {
                if let Effect::SeekTo(to) = effect {
                    prop_assert!(to.get() >= 0.0, "seek to {to:?}");
                }
            }

            // A refused command changes nothing.
            if only_rejection(&out) {
                prop_assert_eq!((p.phase(), p.current_index()), before);
            }

            // Ticks never act while nothing runs, and never go backwards.
            if matches!(input, Input::Tick { .. }) {
                if !is_running(before.0) {
                    prop_assert!(out.effects.is_empty() && out.events.is_empty());
                    prop_assert_eq!(p.phase(), before.0);
                }
                // A tick moves to the next song, or to the song the playhead is in.
                if let (Some(a), Some(b), Input::Tick { position, .. }) = (before.1, p.current_index(), input) {
                    let under_playhead = p.current().is_some_and(|song| song.window.contains(position));
                    prop_assert!(b == a || b == a + 1 || under_playhead);
                }
            }

            // At most one hand-over per step, and never closer than the debounce.
            let hand_overs = out
                .events
                .iter()
                .filter(|e| matches!(e, Event::HandOverStarted { .. }))
                .count();
            prop_assert!(hand_overs <= 1);
            if hand_overs == 1 {
                if let Some(previous) = last_hand_over {
                    prop_assert!(now - previous >= 1.0, "hand-overs {previous} and {now}");
                }
                last_hand_over = Some(now);
            }

            // Finishing or a hard stop always halts playback in the same step.
            let halted = out.events.iter().any(|e| {
                matches!(e, Event::PerformanceFinished | Event::HardStopReached { .. })
            });
            if halted {
                prop_assert_eq!(out.effects.last(), Some(&Effect::Pause));
            }
        }
    }

    /// Playing straight through plays every song once, in order, and then finishes.
    #[test]
    fn playing_through_visits_every_song_once(songs in setlist(1, false)) {
        let ends: Vec<f64> = songs.iter().map(|s| s.window.end().get()).collect();
        let mut p = Performance::new(songs.clone(), Flags::default(), HandOverPolicy::default());
        p.step(Input::Play);

        let mut events = Vec::new();
        let mut now = 0.0;
        for (index, end) in ends.iter().enumerate() {
            now += 10.0;
            // The clock reaches the end of this song a little early and again later.
            for position in [end - 0.01, *end] {
                now += 0.02;
                events.extend(p.step(Input::Tick { now: t(now), position: t(position) }).events);
            }
            if let Some(next) = songs.get(index + 1) {
                now += 0.02;
                events.extend(p.step(Input::Tick { now: t(now), position: next.window.start() }).events);
            }
        }

        let started: Vec<String> = events.iter().filter_map(|e| match e {
            Event::HandOverStarted { to, .. } => Some(to.clone()),
            _ => None,
        }).collect();
        let expected: Vec<String> = songs.iter().skip(1).map(|s| s.song_id.clone()).collect();
        prop_assert_eq!(started, expected);
        prop_assert_eq!(p.phase(), Phase::Finished);
        prop_assert_eq!(events.iter().filter(|e| **e == Event::PerformanceFinished).count(), 1);
    }

    /// A finished performance stays finished however long the clock runs.
    #[test]
    fn finished_stays_finished_until_a_command(
        songs in setlist(1, true),
        ticks in prop::collection::vec((0.0..5.0_f64, 0.0..200.0_f64), 0..60),
    ) {
        let mut p = Performance::new(songs, Flags::default(), HandOverPolicy::default());
        p.step(Input::Play);
        let mut now = 0.0;
        // Run to the end: the clock sits at the end of the current song, two ticks
        // per song (hand-over, then completion), a Play after every hard stop.
        for _ in 0..40 {
            let Some(end) = p.current().map(|song| song.window.end()) else { break };
            now += 3.0;
            p.step(Input::Tick { now: t(now), position: end });
            if let Some(start) = p.current().map(|song| song.window.start()) {
                now += 0.02;
                p.step(Input::Tick { now: t(now), position: start });
            }
            if p.phase() == Phase::HardStopped {
                p.step(Input::Play);
            }
        }
        prop_assert_eq!(p.phase(), Phase::Finished);
        for (dt, position) in ticks {
            now += dt;
            let out = p.step(Input::Tick { now: t(now), position: t(position) });
            prop_assert!(out.effects.is_empty() && out.events.is_empty());
            prop_assert_eq!(p.phase(), Phase::Finished);
        }
    }
}
