use performance::{Input, Phase};
use reaper_port::{FakeReaper, Marker, ReaperPort, Region, Transport};
use shared_kernel::{Seconds, SongId};

use super::TimerLoop;

fn seconds(value: f64) -> Seconds {
    Seconds::new(value).unwrap_or(Seconds::ZERO)
}

fn region(id: &str, start: f64, end: f64) -> Region {
    Region {
        id: SongId::new(id),
        name: id.to_string(),
        start: seconds(start),
        end: seconds(end),
    }
}

fn marker(name: &str, position: f64) -> Marker {
    Marker {
        name: name.to_string(),
        position: seconds(position),
    }
}

fn fake(regions: Vec<Region>, markers: Vec<Marker>) -> FakeReaper {
    FakeReaper::new(
        regions,
        markers,
        performance::TempoMap::constant(
            shared_kernel::Bpm::FALLBACK,
            performance::TimeSignature::common(),
        ),
    )
}

fn two_songs() -> Vec<Region> {
    vec![region("A", 0.0, 10.0), region("B", 10.0, 20.0)]
}

/// Lets 50 ms pass and ticks, for `count` ticks.
fn run(timer_loop: &mut TimerLoop<FakeReaper>, count: usize) {
    for _ in 0..count {
        timer_loop.port_mut().advance(seconds(0.05));
        timer_loop.tick();
    }
}

#[test]
fn play_starts_the_transport_and_the_end_of_a_song_hands_over() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    assert_eq!(timer_loop.phase(), Phase::Idle);
    timer_loop.command(Input::Play);
    assert_eq!(timer_loop.port_mut().transport(), Transport::Playing);
    assert_eq!(timer_loop.phase(), Phase::Playing);
    run(&mut timer_loop, 210);
    assert_eq!(timer_loop.phase(), Phase::Playing);
    assert!(timer_loop.port_mut().position().get() > 10.0);
}

#[test]
fn a_hard_stop_marker_halts_at_the_end_of_the_song() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![marker("!1008", 9.0)]));
    timer_loop.command(Input::Play);
    run(&mut timer_loop, 210);
    assert_eq!(timer_loop.phase(), Phase::HardStopped);
    assert_eq!(timer_loop.port_mut().transport(), Transport::Paused);
}

#[test]
fn pause_goes_to_reaper() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    timer_loop.command(Input::Pause);
    assert_eq!(timer_loop.phase(), Phase::Paused);
    assert_eq!(timer_loop.port_mut().transport(), Transport::Paused);
}

#[test]
fn an_unchanged_project_is_not_read_again_and_the_show_goes_on() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    run(&mut timer_loop, 20);
    assert_eq!(timer_loop.rebuilds(), 0);
    assert_eq!(timer_loop.phase(), Phase::Playing);
}

#[test]
fn an_edit_that_leaves_the_songs_alone_does_not_restart_the_performance() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    timer_loop.port_mut().set_ext_state("note", "key", "value");
    run(&mut timer_loop, 5);
    assert_eq!(timer_loop.rebuilds(), 0);
    assert_eq!(timer_loop.phase(), Phase::Playing);
}

#[test]
fn an_edit_that_changes_the_songs_starts_over_with_the_new_ones() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    timer_loop
        .port_mut()
        .replace_regions(vec![region("A", 0.0, 10.0)]);
    run(&mut timer_loop, 1);
    assert_eq!(timer_loop.rebuilds(), 1);
    assert_eq!(timer_loop.phase(), Phase::Idle);
}

#[test]
fn a_region_without_length_is_not_a_song() {
    let mut timer_loop = TimerLoop::new(fake(vec![region("A", 5.0, 5.0)], Vec::new()));
    timer_loop.command(Input::Play);
    assert_eq!(timer_loop.phase(), Phase::Idle);
}

#[test]
fn a_link_play_starts_the_transport_and_shows_in_the_live_state() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![]));
    let outcome = timer_loop.link_command(protocol::Command::Play);
    timer_loop.tick();
    assert_eq!(outcome, protocol::message::Outcome::Done);
    assert_eq!(timer_loop.live().phase, protocol::Phase::Playing);
}

#[test]
fn a_link_seek_is_counted_from_the_start_of_the_current_song_and_live_shows_the_timeline() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![]));
    timer_loop.command(Input::Next);
    timer_loop.link_command(protocol::Command::Seek {
        position: 3.0,
        count_in: false,
    });
    let start = timer_loop.catalog().songs[1].start;
    assert!((timer_loop.live().position - start - 3.0).abs() < 1e-9);
}

#[test]
fn a_link_toggle_flips_the_setting() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![]));
    let before = timer_loop.live().autoplay;
    timer_loop.link_command(protocol::Command::ToggleAutoResume);
    assert_ne!(timer_loop.live().autoplay, before);
}

#[test]
fn a_counted_in_seek_holds_on_the_cue_then_plays_and_gives_the_setting_back() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![]));
    timer_loop.link_command(protocol::Command::ToggleCountInOnMarker);
    timer_loop.command(Input::Play);
    run(&mut timer_loop, 20);
    let done = timer_loop.link_command(protocol::Command::Seek {
        position: 6.0,
        count_in: true,
    });
    assert!(matches!(done, protocol::message::Outcome::Done));
    assert_eq!(timer_loop.phase(), Phase::CountingIn);
    assert!(timer_loop.port_mut().count_in());
    run(&mut timer_loop, 60);
    assert_eq!(timer_loop.phase(), Phase::CountingIn);
    assert!((timer_loop.port_mut().position().get() - 6.0).abs() < 1e-9);
    run(&mut timer_loop, 40);
    assert_eq!(timer_loop.phase(), Phase::Playing);
    assert!(timer_loop.port_mut().position().get() > 6.0);
}

#[test]
fn the_catalog_lists_the_songs_and_only_the_cues_with_a_label() {
    let markers = vec![
        marker("Chorus", 4.0),
        marker("!1008", 9.0),
        marker("Bridge !length:8", 6.0),
    ];
    let timer_loop = TimerLoop::new(fake(two_songs(), markers));
    let catalog = timer_loop.catalog();
    let names: Vec<&str> = catalog.songs.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["A", "B"]);
    let first = &catalog.songs[0];
    assert!(first.hard_stop);
    assert_eq!(first.length, Some(8.0));
    let cues: Vec<&str> = catalog.cues.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(cues, ["Chorus", "Bridge !length:8"]);
}

#[test]
fn live_names_the_next_song_and_has_none_after_the_last() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    assert_eq!(timer_loop.live().next_song, Some(1));
    timer_loop.command(Input::Next);
    assert_eq!(timer_loop.live().next_song, None);
}

#[test]
fn the_current_song_is_an_index_into_the_catalog() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    assert_eq!(timer_loop.live().current_song, Some(0));
    timer_loop.command(Input::Next);
    assert_eq!(timer_loop.live().current_song, Some(1));
}

#[test]
fn the_catalog_revision_rises_only_when_the_project_content_changes() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    let first = timer_loop.catalog().revision;
    run(&mut timer_loop, 3);
    assert_eq!(timer_loop.catalog().revision, first);
    timer_loop
        .port_mut()
        .replace_regions(vec![region("A", 0.0, 10.0)]);
    run(&mut timer_loop, 1);
    assert_eq!(timer_loop.catalog().revision, first + 1);
    assert_eq!(timer_loop.catalog().songs.len(), 1);
}

const FRIDAY: &str = r#"[{"id":"friday","name":"Friday","revision":2,"entries":[
    {"id":0,"song_id":"B"},{"id":1,"song_id":"gone"},{"id":2,"song_id":"A"},{"id":3,"song_id":"B"}]}]"#;

fn with_setlist(text: &str, active: &str) -> TimerLoop<FakeReaper> {
    let mut port = fake(two_songs(), Vec::new());
    port.set_ext_state("RC2", "setlists", text);
    port.set_ext_state("RC2", "active_setlist", active);
    TimerLoop::new(port)
}

fn song_names(timer_loop: &TimerLoop<FakeReaper>) -> Vec<&str> {
    let songs = &timer_loop.catalog().songs;
    songs.iter().map(|song| song.name.as_str()).collect()
}

#[test]
fn the_played_setlist_sets_the_order_and_skips_songs_the_project_lost() {
    let timer_loop = with_setlist(FRIDAY, "friday");
    assert_eq!(song_names(&timer_loop), ["B", "A", "B"]);
    assert_eq!(
        timer_loop.catalog().active_setlist.as_deref(),
        Some("friday")
    );
    assert_eq!(timer_loop.catalog().setlists.len(), 1);
    assert_eq!(timer_loop.catalog().setlists[0].entries.len(), 4);
}

#[test]
fn the_performance_plays_the_setlist_order() {
    let mut timer_loop = with_setlist(FRIDAY, "friday");
    assert_eq!(timer_loop.live().current_song, Some(0));
    timer_loop.link_command(protocol::Command::Next);
    assert_eq!(timer_loop.live().current_song, Some(1));
    // The second entry is song A, which starts at the top of the timeline.
    assert_eq!(timer_loop.port_mut().position().get(), 0.0);
}

#[test]
fn without_a_played_setlist_the_songs_keep_timeline_order() {
    for active in ["", "unknown"] {
        let timer_loop = with_setlist(FRIDAY, active);
        assert_eq!(song_names(&timer_loop), ["A", "B"]);
        assert_eq!(timer_loop.catalog().active_setlist, None);
        assert_eq!(timer_loop.catalog().setlists.len(), 1);
    }
}

#[test]
fn a_setlist_that_does_not_hold_together_or_is_not_json_is_left_out() {
    let repeated = r#"[{"id":"x","name":"X","revision":0,"entries":[
        {"id":1,"song_id":"A"},{"id":1,"song_id":"B"}]},
        {"id":"y","name":" ","revision":0,"entries":[]}]"#;
    for text in [repeated, "not json", "{}"] {
        let timer_loop = with_setlist(text, "x");
        assert!(timer_loop.catalog().setlists.is_empty(), "{text}");
        assert_eq!(song_names(&timer_loop), ["A", "B"]);
    }
}

#[test]
fn editing_the_setlist_raises_its_revision_and_leaves_the_content_revision_alone() {
    let mut timer_loop = with_setlist(FRIDAY, "friday");
    let content = timer_loop.catalog().revision;
    let setlists = timer_loop.catalog().setlist_revision;
    run(&mut timer_loop, 3);
    assert_eq!(timer_loop.catalog().setlist_revision, setlists);
    timer_loop
        .port_mut()
        .set_ext_state("RC2", "active_setlist", "");
    run(&mut timer_loop, 1);
    assert_eq!(timer_loop.catalog().setlist_revision, setlists + 1);
    assert_eq!(song_names(&timer_loop), ["A", "B"]);
    // The songs listed changed with the order, so the content revision moved too.
    assert_eq!(timer_loop.catalog().revision, content + 1);
}

#[test]
fn what_the_performance_does_is_handed_out_once_in_order() {
    use protocol::WireEvent;
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    assert_eq!(
        timer_loop.take_events(),
        vec![WireEvent::PerformanceStarted]
    );
    assert_eq!(timer_loop.take_events(), Vec::new());
}

#[test]
fn a_refused_link_command_becomes_an_event() {
    use protocol::{Command, WireEvent};
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.link_command(Command::ToggleRecordArm);
    assert!(matches!(
        timer_loop.take_events().as_slice(),
        [WireEvent::CommandRejected { .. }]
    ));
}

#[test]
fn a_hand_over_is_reported_when_a_song_ends() {
    use protocol::WireEvent;
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    timer_loop.take_events();
    run(&mut timer_loop, 220);
    let events = timer_loop.take_events();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WireEvent::HandOverStarted { .. })),
        "{events:?}"
    );
}

fn save(id: &str, songs: &[&str], expected_revision: u64) -> protocol::Command {
    protocol::Command::SaveSetlist {
        id: id.into(),
        name: "Saturday".into(),
        entries: songs
            .iter()
            .zip(1..)
            .map(|(song_id, id)| protocol::EntryInfo {
                id,
                song_id: (*song_id).into(),
            })
            .collect(),
        expected_revision,
    }
}

#[test]
fn a_saved_setlist_is_stored_in_the_project_and_shows_in_the_catalog() {
    use protocol::message::Outcome;
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    let before = timer_loop.catalog().setlist_revision;
    assert_eq!(
        timer_loop.link_command(save("sat", &["B", "A"], 0)),
        Outcome::Done
    );
    let setlists = &timer_loop.catalog().setlists;
    assert_eq!(setlists.len(), 1);
    assert_eq!(
        (setlists[0].name.as_str(), setlists[0].revision),
        ("Saturday", 1)
    );
    assert_eq!(setlists[0].entries.len(), 2);
    assert_eq!(timer_loop.catalog().setlist_revision, before + 1);
    let stored = timer_loop.port_mut().ext_state("RC2", "setlists");
    assert!(stored.is_some_and(|text| text.contains("Saturday")));
}

#[test]
fn saving_again_needs_the_revision_that_was_seen() {
    use protocol::message::Outcome;
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.link_command(save("sat", &["A"], 0));
    assert!(matches!(
        timer_loop.link_command(save("sat", &["B"], 0)),
        Outcome::Rejected { .. }
    ));
    assert_eq!(timer_loop.catalog().setlists[0].entries[0].song_id, "A");
    assert_eq!(
        timer_loop.link_command(save("sat", &["B"], 1)),
        Outcome::Done
    );
    assert_eq!(timer_loop.catalog().setlists[0].entries[0].song_id, "B");
    assert_eq!(timer_loop.catalog().setlists[0].revision, 2);
}

#[test]
fn a_setlist_that_cannot_hold_together_is_refused_and_reported() {
    use protocol::message::Outcome;
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    let mut nameless = save("sat", &["A"], 0);
    if let protocol::Command::SaveSetlist { name, .. } = &mut nameless {
        name.clear();
    }
    assert!(matches!(
        timer_loop.link_command(nameless),
        Outcome::Rejected { .. }
    ));
    assert!(timer_loop.catalog().setlists.is_empty());
    assert_eq!(timer_loop.take_events().len(), 1);
}

#[test]
fn the_catalog_lists_every_song_in_timeline_order_whatever_is_played() {
    let timer_loop = with_setlist(FRIDAY, "friday");
    let names: Vec<_> = timer_loop
        .catalog()
        .project_songs
        .iter()
        .map(|song| song.name.as_str())
        .collect();
    assert_eq!(names, ["A", "B"]);
    assert_eq!(timer_loop.catalog().songs.len(), 3);
}

#[test]
fn the_played_setlist_can_be_chosen_and_cleared_and_an_unknown_one_is_refused() {
    use protocol::Command::SetActiveSetlist;
    use protocol::message::Outcome;
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.link_command(save("sat", &["B", "A"], 0));
    let choose = |id: Option<&str>| SetActiveSetlist {
        id: id.map(str::to_owned),
    };
    assert!(matches!(
        timer_loop.link_command(choose(Some("gone"))),
        Outcome::Rejected { .. }
    ));
    assert_eq!(timer_loop.catalog().active_setlist, None);
    assert_eq!(timer_loop.link_command(choose(Some("sat"))), Outcome::Done);
    assert_eq!(timer_loop.catalog().active_setlist.as_deref(), Some("sat"));
    assert_eq!(timer_loop.catalog().songs[0].id, "B");
    assert_eq!(
        timer_loop
            .port_mut()
            .ext_state("RC2", "active_setlist")
            .as_deref(),
        Some("sat")
    );
    assert_eq!(timer_loop.link_command(choose(None)), Outcome::Done);
    assert_eq!(timer_loop.catalog().active_setlist, None);
    assert_eq!(timer_loop.catalog().songs[0].id, "A");
}

#[test]
fn a_project_gets_an_id_that_stays_and_is_in_the_catalog() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    let id = timer_loop.catalog().project_id.clone();
    assert!(id.starts_with("project-"));
    assert_eq!(
        timer_loop.port_mut().ext_state("RC2", "project_id"),
        Some(id.clone())
    );
    run(&mut timer_loop, 3);
    assert_eq!(timer_loop.catalog().project_id, id);
}

#[test]
fn the_id_v1_stored_is_adopted() {
    let mut port = fake(two_songs(), Vec::new());
    port.set_ext_state("ReaperControl", "ProjectId", "project-1754-abc");
    let timer_loop = TimerLoop::new(port);
    assert_eq!(timer_loop.catalog().project_id, "project-1754-abc");
}

#[test]
fn the_id_v1_stored_is_adopted_from_a_saved_project_too() {
    let mut port = fake(two_songs(), Vec::new());
    port.set_ext_state("REAPERCONTROL", "PROJECTID", "project-1754-abc");
    let timer_loop = TimerLoop::new(port);
    assert_eq!(timer_loop.catalog().project_id, "project-1754-abc");
}

#[test]
fn an_id_that_cannot_name_a_file_is_replaced() {
    let mut port = fake(two_songs(), Vec::new());
    port.set_ext_state("RC2", "project_id", "../escape");
    let timer_loop = TimerLoop::new(port);
    assert!(timer_loop.catalog().project_id.starts_with("project-"));
}
