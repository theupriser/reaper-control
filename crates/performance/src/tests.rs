#![allow(clippy::unwrap_used, clippy::expect_used)]

use shared_kernel::{InvalidValue, Seconds};

use super::*;

fn t(value: f64) -> Seconds {
    Seconds::new(value).unwrap()
}

fn song(id: &str, start: f64, end: f64, hard_stop: bool) -> PlannedSong {
    PlannedSong {
        song_id: id.to_string(),
        window: SongWindow::new(t(start), t(end)).unwrap(),
        hard_stop,
        hard_stop_marker: None,
    }
}

/// A 0-10 and B 10-20 touch, C 30-40 has a hard stop, D 50-60 is last.
fn setlist() -> Vec<PlannedSong> {
    vec![
        song("A", 0.0, 10.0, false),
        song("B", 10.0, 20.0, false),
        song("C", 30.0, 40.0, true),
        song("D", 50.0, 60.0, false),
    ]
}

fn performance(flags: Flags) -> Performance {
    Performance::new(setlist(), flags, HandOverPolicy::default())
}

fn tick(performance: &mut Performance, now: f64, position: f64) -> Output {
    performance.step(Input::Tick {
        now: t(now),
        position: t(position),
    })
}

fn playing_at_song(index: usize) -> Performance {
    let mut p = performance(Flags::default());
    p.step(Input::Play);
    for _ in 0..index {
        p.step(Input::Next);
    }
    p.step(Input::Play);
    p
}

fn rejected(why: Rejection) -> Vec<Event> {
    vec![Event::CommandRejected(why)]
}

#[test]
fn play_from_idle_seeks_to_the_start_and_plays() {
    let mut p = performance(Flags::default());
    let out = p.step(Input::Play);
    assert_eq!(out.effects, vec![Effect::SeekTo(t(0.0)), Effect::Play]);
    assert_eq!(out.events, vec![Event::PerformanceStarted]);
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn empty_setlist_cannot_play() {
    let mut p = Performance::new(vec![], Flags::default(), HandOverPolicy::default());
    let out = p.step(Input::Play);
    assert!(out.effects.is_empty());
    assert_eq!(out.events, rejected(Rejection::NothingToPlay));
    assert_eq!(p.phase(), Phase::Idle);
}

#[test]
fn pause_then_play_resumes_where_it_was() {
    let mut p = performance(Flags::default());
    p.step(Input::Play);
    assert_eq!(p.step(Input::Pause).effects, vec![Effect::Pause]);
    assert_eq!(p.phase(), Phase::Paused);
    assert_eq!(p.step(Input::Play).effects, vec![Effect::Play]);
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn nothing_happens_before_the_lead() {
    let mut p = performance(Flags::default());
    p.step(Input::Play);
    assert_eq!(tick(&mut p, 1.0, 9.9), Output::default());
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn contiguous_hand_over_does_not_seek() {
    let mut p = performance(Flags::default());
    p.step(Input::Play);
    let out = tick(&mut p, 9.99, 9.99);
    assert!(out.effects.is_empty());
    assert_eq!(
        out.events,
        vec![Event::HandOverStarted {
            from: "A".into(),
            to: "B".into()
        }]
    );
    assert_eq!(p.phase(), Phase::HandingOver);
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("B"));
}

#[test]
fn hand_over_over_a_gap_seeks_to_the_next_start() {
    let mut p = playing_at_song(1);
    let out = tick(&mut p, 20.0, 19.99);
    assert_eq!(out.effects, vec![Effect::SeekTo(t(30.0))]);
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("C"));
}

#[test]
fn hand_over_completes_when_the_position_is_in_the_next_song() {
    let mut p = playing_at_song(1);
    tick(&mut p, 20.0, 19.99);
    assert_eq!(tick(&mut p, 20.03, 19.99).events, vec![]);
    assert_eq!(p.phase(), Phase::HandingOver);
    let out = tick(&mut p, 20.06, 30.02);
    assert_eq!(
        out.events,
        vec![Event::HandOverCompleted {
            song_id: "C".into()
        }]
    );
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn one_song_end_fires_one_hand_over() {
    let mut p = performance(Flags::default());
    p.step(Input::Play);
    let mut started = 0;
    for step in 0..10 {
        let out = tick(&mut p, 9.99 + f64::from(step) * 0.03, 9.99);
        started += out
            .events
            .iter()
            .filter(|e| matches!(e, Event::HandOverStarted { .. }))
            .count();
    }
    assert_eq!(started, 1);
}

#[test]
fn a_hand_over_right_after_another_is_ignored() {
    let songs = vec![
        song("A", 0.0, 10.0, false),
        song("B", 10.0, 10.01, false),
        song("C", 10.01, 20.0, false),
    ];
    let mut p = Performance::new(songs, Flags::default(), HandOverPolicy::default());
    p.step(Input::Play);
    tick(&mut p, 9.99, 9.99);
    tick(&mut p, 10.02, 10.0);
    let out = tick(&mut p, 10.05, 10.005);
    assert_eq!(out, Output::default());
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("B"));
    let out = tick(&mut p, 11.0, 10.005);
    assert_eq!(out.events.len(), 1);
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("C"));
}

#[test]
fn hard_stop_halts_at_the_song_end() {
    let mut p = playing_at_song(2);
    let out = tick(&mut p, 40.0, 39.99);
    assert_eq!(out.effects, vec![Effect::Pause]);
    assert_eq!(
        out.events,
        vec![Event::HardStopReached {
            song_id: "C".into()
        }]
    );
    assert_eq!(p.phase(), Phase::HardStopped);
    assert_eq!(tick(&mut p, 41.0, 40.0), Output::default());
}

#[test]
fn hard_stop_honours_a_shorter_length() {
    let songs = vec![song("A", 0.0, 6.0, true), song("B", 10.0, 20.0, false)];
    let mut p = Performance::new(songs, Flags::default(), HandOverPolicy::default());
    p.step(Input::Play);
    assert_eq!(tick(&mut p, 5.0, 5.9).effects, vec![]);
    assert_eq!(tick(&mut p, 6.0, 5.99).effects, vec![Effect::Pause]);
    assert_eq!(p.phase(), Phase::HardStopped);
}

#[test]
fn play_after_a_hard_stop_goes_into_the_next_song() {
    let mut p = playing_at_song(2);
    tick(&mut p, 40.0, 39.99);
    let out = p.step(Input::Play);
    assert_eq!(out.effects, vec![Effect::SeekTo(t(50.0)), Effect::Play]);
    assert_eq!(p.phase(), Phase::Playing);
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("D"));
}

#[test]
fn hard_stop_on_the_last_song_finishes() {
    let songs = vec![song("A", 0.0, 10.0, true)];
    let mut p = Performance::new(songs, Flags::default(), HandOverPolicy::default());
    p.step(Input::Play);
    let out = tick(&mut p, 10.0, 9.99);
    assert_eq!(
        out.events,
        vec![
            Event::HardStopReached {
                song_id: "A".into()
            },
            Event::PerformanceFinished
        ]
    );
    assert_eq!(p.phase(), Phase::Finished);
}

#[test]
fn the_last_song_ending_finishes_and_pauses() {
    let mut p = playing_at_song(3);
    let out = tick(&mut p, 60.0, 59.99);
    assert_eq!(out.effects, vec![Effect::Pause]);
    assert_eq!(out.events, vec![Event::PerformanceFinished]);
    assert_eq!(p.phase(), Phase::Finished);
}

#[test]
fn finished_stays_finished_until_play_restarts() {
    let mut p = playing_at_song(3);
    tick(&mut p, 60.0, 59.99);
    assert_eq!(tick(&mut p, 61.0, 60.0), Output::default());
    assert_eq!(p.step(Input::Pause), Output::default());
    assert_eq!(p.phase(), Phase::Finished);
    let out = p.step(Input::Play);
    assert_eq!(out.effects, vec![Effect::SeekTo(t(0.0)), Effect::Play]);
    assert_eq!(out.events, vec![Event::PerformanceStarted]);
    assert_eq!(p.current_index(), Some(0));
}

#[test]
fn next_with_autoplay_seeks_and_plays() {
    let mut p = performance(Flags {
        autoplay: true,
        count_in: false,
    });
    p.step(Input::Play);
    let out = p.step(Input::Next);
    assert_eq!(
        out.effects,
        vec![Effect::Pause, Effect::SeekTo(t(10.0)), Effect::Play]
    );
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn next_without_autoplay_seeks_and_waits() {
    let mut p = performance(Flags::default());
    p.step(Input::Play);
    let out = p.step(Input::Next);
    assert_eq!(out.effects, vec![Effect::Pause, Effect::SeekTo(t(10.0))]);
    assert_eq!(p.phase(), Phase::Paused);
}

#[test]
fn next_and_previous_stop_at_the_ends_of_the_setlist() {
    let mut p = performance(Flags::default());
    assert_eq!(
        p.step(Input::Previous).events,
        rejected(Rejection::NoPreviousSong)
    );
    let mut last = playing_at_song(3);
    assert_eq!(
        last.step(Input::Next).events,
        rejected(Rejection::NoNextSong)
    );
}

#[test]
fn restart_goes_back_to_the_start_of_the_song() {
    let mut p = playing_at_song(1);
    let out = p.step(Input::RestartSong);
    assert_eq!(out.effects, vec![Effect::Pause, Effect::SeekTo(t(10.0))]);
    assert_eq!(p.current_index(), Some(1));
}

#[test]
fn go_to_song_jumps_to_that_songs_start_and_autoplay_decides_whether_it_plays() {
    let mut waiting = playing_at_song(0);
    let out = waiting.step(Input::GoToSong { index: 2 });
    assert_eq!(out.effects, vec![Effect::Pause, Effect::SeekTo(t(30.0))]);
    assert_eq!(waiting.current_index(), Some(2));

    let mut playing = performance(Flags {
        autoplay: true,
        ..Flags::default()
    });
    playing.step(Input::Play);
    let out = playing.step(Input::GoToSong { index: 1 });
    assert_eq!(
        out.effects,
        vec![Effect::Pause, Effect::SeekTo(t(10.0)), Effect::Play]
    );
    assert_eq!(playing.phase(), Phase::Playing);
}

#[test]
fn a_song_click_does_not_start_playback_that_was_not_running() {
    let flags = Flags {
        autoplay: true,
        count_in: false,
    };
    let mut idle = performance(flags);
    let out = idle.step(Input::GoToSong { index: 1 });
    assert_eq!(out.effects, vec![Effect::Pause, Effect::SeekTo(t(10.0))]);
    assert_eq!(idle.phase(), Phase::Paused);

    let mut paused = performance(flags);
    paused.step(Input::Play);
    paused.step(Input::Pause);
    let out = paused.step(Input::GoToSong { index: 2 });
    assert_eq!(out.effects, vec![Effect::Pause, Effect::SeekTo(t(30.0))]);
    assert_eq!(paused.phase(), Phase::Paused);
}

#[test]
fn go_to_song_refuses_a_song_that_is_not_in_the_setlist() {
    let mut p = playing_at_song(0);
    assert_eq!(
        p.step(Input::GoToSong { index: 40 }).events,
        rejected(Rejection::NoSuchSong)
    );
    assert_eq!(p.current_index(), Some(0));
}

#[test]
fn seek_stays_inside_the_current_song() {
    let mut p = playing_at_song(0);
    let out = p.step(Input::Seek { position: t(4.0) });
    assert_eq!(
        out.effects,
        vec![Effect::Pause, Effect::SeekTo(t(4.0)), Effect::Play]
    );
    assert_eq!(p.phase(), Phase::Playing);
    let out = p.step(Input::Seek { position: t(14.0) });
    assert!(out.effects.is_empty());
    assert_eq!(out.events, rejected(Rejection::OutsideSong));
}

#[test]
fn seek_while_paused_resumes_only_with_auto_resume() {
    let mut waiting = playing_at_song(0);
    waiting.step(Input::Pause);
    let out = waiting.step(Input::Seek { position: t(4.0) });
    assert_eq!(out.effects, vec![Effect::SeekTo(t(4.0))]);
    assert_eq!(waiting.phase(), Phase::Paused);

    let mut resuming = performance(Flags {
        autoplay: true,
        count_in: false,
    });
    resuming.step(Input::Play);
    resuming.step(Input::Pause);
    let out = resuming.step(Input::Seek { position: t(4.0) });
    assert_eq!(out.effects, vec![Effect::SeekTo(t(4.0)), Effect::Play]);
    assert_eq!(resuming.phase(), Phase::Playing);
}

#[test]
fn seek_outside_the_song_while_paused_does_not_resume() {
    let mut p = performance(Flags {
        autoplay: true,
        count_in: false,
    });
    p.step(Input::Play);
    p.step(Input::Pause);
    let out = p.step(Input::Seek { position: t(14.0) });
    assert!(out.effects.is_empty());
    assert_eq!(p.phase(), Phase::Paused);
}

#[test]
fn cue_jump_counts_in_when_enabled() {
    let mut p = performance(Flags {
        autoplay: false,
        count_in: true,
    });
    p.step(Input::Play);
    let out = p.step(Input::SeekCue { position: t(6.0) });
    assert_eq!(
        out.effects,
        vec![
            Effect::Pause,
            Effect::SeekTo(t(6.0)),
            Effect::SetCountIn(true),
            Effect::Play
        ]
    );
    assert_eq!(p.phase(), Phase::CountingIn);
    assert_eq!(tick(&mut p, 1.0, 6.0), Output::default());
    let out = tick(&mut p, 1.1, 6.05);
    assert_eq!(out.effects, vec![Effect::SetCountIn(false)]);
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn cue_jump_without_the_flag_is_a_plain_seek() {
    let mut p = playing_at_song(0);
    let out = p.step(Input::SeekCue { position: t(6.0) });
    assert_eq!(
        out.effects,
        vec![Effect::Pause, Effect::SeekTo(t(6.0)), Effect::Play]
    );
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn manual_navigation_never_counts_in() {
    let mut p = performance(Flags {
        autoplay: true,
        count_in: true,
    });
    p.step(Input::Play);
    let out = p.step(Input::Next);
    assert!(!out.effects.contains(&Effect::SetCountIn(true)));
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn pausing_a_count_in_cancels_it() {
    let mut p = performance(Flags {
        autoplay: false,
        count_in: true,
    });
    p.step(Input::Play);
    p.step(Input::SeekCue { position: t(6.0) });
    let out = p.step(Input::Pause);
    assert_eq!(out.effects, vec![Effect::SetCountIn(false), Effect::Pause]);
    assert_eq!(p.phase(), Phase::Paused);
}

#[test]
fn a_flag_event_is_sent_only_when_it_changes() {
    let mut p = performance(Flags::default());
    let on = Input::SetFlag {
        flag: Flag::Autoplay,
        enabled: true,
    };
    assert_eq!(
        p.step(on).events,
        vec![Event::FlagChanged {
            flag: Flag::Autoplay,
            enabled: true
        }]
    );
    assert_eq!(p.step(on), Output::default());
    assert!(p.flags().autoplay);
}

#[test]
fn a_window_needs_an_end_after_the_start() {
    assert_eq!(
        SongWindow::new(t(5.0), t(5.0)),
        Err(InvalidValue::OutOfRange)
    );
    assert_eq!(
        SongWindow::new(t(5.0), t(1.0)),
        Err(InvalidValue::OutOfRange)
    );
}

fn observe(p: &mut Performance, playing: bool, times: usize) {
    for _ in 0..times {
        let out = p.step(Input::Observed { playing });
        assert!(out.effects.is_empty() && out.events.is_empty());
    }
}

#[test]
fn a_pause_made_in_reaper_is_followed_after_a_few_ticks() {
    let mut p = playing_at_song(1);
    observe(&mut p, false, 2);
    assert_eq!(p.phase(), Phase::Playing);
    observe(&mut p, false, 1);
    assert_eq!(p.phase(), Phase::Paused);
}

#[test]
fn a_play_made_in_reaper_is_followed_after_a_few_ticks() {
    let mut p = playing_at_song(1);
    p.step(Input::Pause);
    observe(&mut p, true, 3);
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn one_stale_reading_after_a_command_does_not_flip_the_phase() {
    let mut p = playing_at_song(1);
    observe(&mut p, false, 2);
    observe(&mut p, true, 1);
    observe(&mut p, false, 2);
    assert_eq!(p.phase(), Phase::Playing);
    p.step(Input::Pause);
    observe(&mut p, true, 2);
    p.step(Input::Play);
    observe(&mut p, true, 2);
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn an_idle_or_hard_stopped_performance_ignores_the_transport() {
    let mut idle = performance(Flags::default());
    observe(&mut idle, true, 10);
    assert_eq!(idle.phase(), Phase::Idle);
}

/// A 0-60 song with a hard stop whose marker lies at 40 (REAPER stops there), then a next song.
fn stopped_by_marker() -> Performance {
    let mut first = song("A", 0.0, 60.0, true);
    first.hard_stop_marker = Some(t(40.0));
    let songs = vec![first, song("B", 70.0, 80.0, false)];
    let mut p = Performance::new(songs, Flags::default(), HandOverPolicy::default());
    p.step(Input::Play);
    p
}

fn observe_once(performance: &mut Performance, playing: bool) {
    performance.step(Input::Observed { playing });
}

#[test]
fn reaper_stopping_at_the_marker_does_not_end_the_song_early() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    for second in 1..=5 {
        observe_once(&mut p, false);
        tick(&mut p, 100.0 + f64::from(second), 40.0);
    }
    assert_eq!(p.phase(), Phase::Playing);
    assert_eq!(p.shown_position(t(105.0), t(40.0)), t(45.0));
}

#[test]
fn the_hard_stop_comes_at_the_end_of_the_length_after_reaper_stopped() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    observe_once(&mut p, false);
    let out = tick(&mut p, 120.0, 40.0);
    assert_eq!(out.effects, vec![Effect::Pause]);
    assert_eq!(
        out.events,
        vec![Event::HardStopReached {
            song_id: "A".into()
        }]
    );
    assert_eq!(p.phase(), Phase::HardStopped);
    assert_eq!(p.shown_position(t(130.0), t(40.0)), t(60.0));
}

#[test]
fn play_after_that_hard_stop_goes_into_the_next_song() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    observe_once(&mut p, false);
    tick(&mut p, 120.0, 40.0);
    let out = p.step(Input::Play);
    assert_eq!(out.effects, vec![Effect::SeekTo(t(70.0)), Effect::Play]);
    assert_eq!(p.shown_position(t(121.0), t(70.0)), t(70.0));
}

#[test]
fn pause_freezes_the_run_on_and_play_runs_it_on_again() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    tick(&mut p, 105.0, 40.0);
    let paused = p.step(Input::Pause);
    assert_eq!(paused, Output::default());
    assert_eq!(p.phase(), Phase::Paused);
    assert_eq!(p.shown_position(t(130.0), t(40.0)), t(45.0));
    tick(&mut p, 130.0, 40.0);
    assert_eq!(p.shown_position(t(130.0), t(40.0)), t(45.0));
    let resumed = p.step(Input::Play);
    assert_eq!(resumed, Output::default());
    assert_eq!(p.phase(), Phase::Playing);
    tick(&mut p, 131.0, 40.0);
    tick(&mut p, 134.0, 40.0);
    assert_eq!(p.shown_position(t(134.0), t(40.0)), t(48.0));
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("A"));
}

#[test]
fn play_while_the_run_on_has_not_reached_its_end_does_not_skip_the_song() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    assert_eq!(p.step(Input::Play), Output::default());
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("A"));
    tick(&mut p, 110.0, 40.0);
    assert_eq!(p.shown_position(t(110.0), t(40.0)), t(50.0));
}

#[test]
fn a_resumed_run_on_still_ends_in_the_hard_stop() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    p.step(Input::Pause);
    p.step(Input::Play);
    tick(&mut p, 200.0, 40.0);
    let out = tick(&mut p, 220.0, 40.0);
    assert_eq!(out.effects, vec![Effect::Pause]);
    assert_eq!(p.phase(), Phase::HardStopped);
}

#[test]
fn a_pause_before_the_marker_stays_a_pause() {
    let mut p = stopped_by_marker();
    for second in 0..4 {
        observe_once(&mut p, false);
        tick(&mut p, 100.0 + f64::from(second), 20.0);
    }
    assert_eq!(p.phase(), Phase::Paused);
    assert_eq!(p.shown_position(t(104.0), t(20.0)), t(20.0));
}

#[test]
fn reaper_playing_again_takes_over_from_the_timeline() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    for _ in 0..3 {
        observe_once(&mut p, true);
    }
    assert_eq!(p.shown_position(t(105.0), t(41.0)), t(41.0));
    assert_eq!(tick(&mut p, 105.0, 41.0), Output::default());
    assert_eq!(p.phase(), Phase::Playing);
}

#[test]
fn a_song_without_a_marker_position_gives_way_to_a_stopped_reaper() {
    let mut p = Performance::new(
        vec![song("A", 0.0, 60.0, true)],
        Flags::default(),
        HandOverPolicy::default(),
    );
    p.step(Input::Play);
    for second in 0..4 {
        observe_once(&mut p, false);
        tick(&mut p, 100.0 + f64::from(second), 40.0);
    }
    assert_eq!(p.phase(), Phase::Paused);
}

#[test]
fn a_seek_after_the_marker_moves_only_the_timeline() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    let out = p.step(Input::Seek { position: t(50.0) });
    assert_eq!(out.effects, vec![Effect::Pause]);
    assert_eq!(out.events, vec![Event::SeekPerformed { to: t(50.0) }]);
    assert_eq!(p.phase(), Phase::Playing);
    assert_eq!(p.shown_position(t(101.0), t(40.0)), t(50.0));
    tick(&mut p, 101.0, 40.0);
    tick(&mut p, 105.0, 40.0);
    assert_eq!(p.shown_position(t(105.0), t(40.0)), t(54.0));
    assert_eq!(p.current().map(|s| s.song_id.as_str()), Some("A"));
}

#[test]
fn a_seek_after_the_marker_while_reaper_still_plays_stops_reaper_and_keeps_time() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, true);
    tick(&mut p, 100.0, 20.0);
    let out = p.step(Input::Seek { position: t(50.0) });
    assert_eq!(out.effects, vec![Effect::Pause]);
    observe_once(&mut p, true);
    observe_once(&mut p, true);
    tick(&mut p, 101.0, 21.0);
    tick(&mut p, 103.0, 22.0);
    assert_eq!(p.shown_position(t(103.0), t(22.0)), t(52.0));
}

#[test]
fn a_timeline_seek_still_ends_in_the_hard_stop_at_the_end_of_the_length() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    p.step(Input::Seek { position: t(50.0) });
    tick(&mut p, 101.0, 40.0);
    let out = tick(&mut p, 111.0, 40.0);
    assert_eq!(out.effects, vec![Effect::Pause]);
    assert_eq!(p.phase(), Phase::HardStopped);
}

#[test]
fn a_seek_before_the_marker_still_moves_reaper() {
    let mut p = stopped_by_marker();
    observe_once(&mut p, false);
    tick(&mut p, 100.0, 40.0);
    let out = p.step(Input::Seek { position: t(10.0) });
    assert_eq!(
        out.effects,
        vec![Effect::Pause, Effect::SeekTo(t(10.0)), Effect::Play]
    );
}

#[test]
fn a_timeline_seek_while_paused_without_auto_resume_stays_paused() {
    let flags = Flags {
        autoplay: false,
        ..Flags::default()
    };
    let mut first = song("A", 0.0, 60.0, true);
    first.hard_stop_marker = Some(t(40.0));
    let mut p = Performance::new(vec![first], flags, HandOverPolicy::default());
    p.step(Input::Play);
    p.step(Input::Pause);
    let out = p.step(Input::Seek { position: t(50.0) });
    assert_eq!(out.effects, vec![Effect::Pause]);
    assert_eq!(p.phase(), Phase::Paused);
    tick(&mut p, 200.0, 20.0);
    assert_eq!(p.shown_position(t(205.0), t(20.0)), t(50.0));
}
