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
fn a_link_play_starts_the_transport_and_shows_in_the_app_state() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![]));
    let outcome = timer_loop.link_command(protocol::Command::Play);
    timer_loop.tick();
    assert_eq!(outcome, protocol::message::Outcome::Done);
    assert_eq!(timer_loop.app_state().phase, protocol::Phase::Playing);
}

#[test]
fn a_link_seek_is_counted_from_the_start_of_the_current_song() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![]));
    timer_loop.command(Input::Next);
    timer_loop.link_command(protocol::Command::Seek {
        position: 3.0,
        count_in: false,
    });
    assert!((timer_loop.app_state().position - 3.0).abs() < 1e-9);
}

#[test]
fn a_link_toggle_flips_the_setting_and_a_count_in_seek_is_refused() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), vec![]));
    let before = timer_loop.app_state().auto_resume;
    timer_loop.link_command(protocol::Command::ToggleAutoResume);
    assert_ne!(timer_loop.app_state().auto_resume, before);
    let refused = timer_loop.link_command(protocol::Command::Seek {
        position: 1.0,
        count_in: true,
    });
    assert!(matches!(
        refused,
        protocol::message::Outcome::Rejected { .. }
    ));
}
