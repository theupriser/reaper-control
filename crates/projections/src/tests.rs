#![allow(clippy::unwrap_used, clippy::expect_used)]

use catalogue::{Cue, Song};
use performance::{Flags, HandOverPolicy, Input, Performance, Phase};
use setlists::{Edit, EntryId, Revision, Setlist, SetlistId};
use shared_kernel::{Seconds, SongId};

use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn secs(value: f64) -> Seconds {
    Seconds::new(value).unwrap()
}

fn song(id: &str, start: f64, end: f64) -> Song {
    Song::new(
        SongId::new(id),
        format!("Song {id}"),
        secs(start),
        secs(end),
    )
    .unwrap()
}

fn setlist(ids: &[&str]) -> Setlist {
    let mut list = Setlist::new(SetlistId::new("main"), "Friday").unwrap();
    for id in ids {
        let revision = list.revision();
        list.edit(
            revision,
            Edit::Add {
                song: SongId::new(*id),
                at: None,
            },
        )
        .unwrap();
    }
    list
}

#[test]
fn plan_applies_length_and_hard_stop_from_the_cues() -> TestResult {
    let songs = [song("A", 0.0, 100.0), song("B", 100.0, 200.0)];
    let cues = [
        Cue::new("!length:30", secs(10.0)),
        Cue::new("!1008", secs(150.0)),
    ];
    let plan = Plan::build(&setlist(&["A", "B"]), &songs, &cues);
    assert!(plan.skipped.is_empty());
    let [a, b] = plan.songs.as_slice() else {
        return Err("two songs".into());
    };
    assert_eq!(a.window.end(), secs(30.0));
    assert!(!a.hard_stop);
    assert_eq!(b.window.end(), secs(200.0));
    assert!(b.hard_stop);
    Ok(())
}

#[test]
fn plan_skips_songs_the_project_lost_and_names_the_entries() {
    let list = setlist(&["A", "gone", "B"]);
    let songs = [song("A", 0.0, 10.0), song("B", 10.0, 20.0)];
    let plan = Plan::build(&list, &songs, &[]);
    assert_eq!(plan.songs.len(), 2);
    assert_eq!(plan.skipped, vec![EntryId::new(1)]);
}

#[test]
fn a_plan_runs_in_the_performance() -> TestResult {
    let songs = [song("A", 0.0, 10.0), song("B", 10.0, 20.0)];
    let plan = Plan::build(&setlist(&["A", "B"]), &songs, &[]);
    let mut run = Performance::new(
        plan.songs.clone(),
        Flags::default(),
        HandOverPolicy::default(),
    );
    run.step(Input::Play);
    let view = PerformanceView::of(&run, &plan.songs);
    assert_eq!(view.phase, Phase::Playing);
    assert_eq!(view.current.as_deref(), Some("A"));
    assert_eq!(view.next.as_deref(), Some("B"));
    assert_eq!(view.index, Some(0));
    Ok(())
}

#[test]
fn the_last_song_has_no_next_and_an_empty_setlist_has_no_current() {
    let songs = [song("A", 0.0, 10.0)];
    let plan = Plan::build(&setlist(&["A"]), &songs, &[]);
    let run = Performance::new(
        plan.songs.clone(),
        Flags::default(),
        HandOverPolicy::default(),
    );
    assert_eq!(PerformanceView::of(&run, &plan.songs).next, None);
    let empty = Performance::new(Vec::new(), Flags::default(), HandOverPolicy::default());
    let view = PerformanceView::of(&empty, &[]);
    assert_eq!((view.current, view.index), (None, None));
}

#[test]
fn setlist_view_resolves_rows_and_keeps_dangling_ones() {
    let songs = [song("A", 5.0, 65.0), song("B", 65.0, 100.0)];
    let cues = [
        Cue::new("!length:20", secs(10.0)),
        Cue::new("!1008", secs(70.0)),
    ];
    let view = SetlistView::of(&setlist(&["A", "gone", "B"]), &songs, &cues);
    assert_eq!(view.name, "Friday");
    assert_eq!(view.revision, Revision::new(3));
    let rows: Vec<_> = view
        .entries
        .iter()
        .map(|row| {
            (
                row.name.clone(),
                row.length,
                row.hard_stop,
                row.is_dangling(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (Some("Song A".into()), Some(secs(20.0)), false, false),
            (None, None, false, true),
            (Some("Song B".into()), Some(secs(35.0)), true, false),
        ]
    );
}

#[test]
fn feed_drops_duplicates_and_late_updates() {
    let mut feed = LiveFeed::new();
    assert_eq!(feed.accept(5, secs(0.0)), Applied::Accepted);
    assert_eq!(feed.accept(5, secs(0.1)), Applied::Ignored);
    assert_eq!(feed.accept(4, secs(0.1)), Applied::Ignored);
    assert_eq!(feed.accept(9, secs(0.2)), Applied::Accepted);
    assert_eq!(feed.sequence(), Some(9));
}

#[test]
fn feed_is_stale_until_it_hears_and_after_a_second_of_silence() {
    let mut feed = LiveFeed::new();
    assert_eq!(feed.freshness(secs(0.0)), Freshness::Stale);
    feed.accept(1, secs(10.0));
    assert_eq!(feed.freshness(secs(10.0)), Freshness::Fresh);
    assert_eq!(feed.freshness(secs(11.0)), Freshness::Fresh);
    assert_eq!(feed.freshness(secs(11.01)), Freshness::Stale);
}

#[test]
fn a_reset_feed_takes_a_restarted_sequence() {
    let mut feed = LiveFeed::new();
    feed.accept(500, secs(0.0));
    assert_eq!(feed.accept(1, secs(1.0)), Applied::Ignored);
    feed.reset();
    assert_eq!(feed.freshness(secs(0.0)), Freshness::Stale);
    assert_eq!(feed.accept(1, secs(2.0)), Applied::Accepted);
}

#[test]
fn player_view_carries_the_feed_state() {
    let run = Performance::new(Vec::new(), Flags::default(), HandOverPolicy::default());
    let mut feed = LiveFeed::new();
    feed.accept(7, secs(3.0));
    let view = PlayerView::new(PerformanceView::of(&run, &[]), secs(12.5), &feed, secs(3.5));
    assert_eq!((view.sequence, view.freshness), (Some(7), Freshness::Fresh));
    assert_eq!(view.position, secs(12.5));
    let late = PlayerView::new(PerformanceView::of(&run, &[]), secs(12.5), &feed, secs(9.0));
    assert_eq!(late.freshness, Freshness::Stale);
}
