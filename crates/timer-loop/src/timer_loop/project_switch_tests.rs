use performance::{Input, Phase};
use protocol::WireEvent;
use reaper_port::{ReaperPort, Transport};

use super::TimerLoop;
use super::tests::{fake, region, run, two_songs};

#[test]
fn switching_to_another_project_replaces_the_songs_even_when_the_change_count_is_the_same() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    let first_revision = timer_loop.catalog().revision;
    timer_loop
        .port_mut()
        .switch_project(vec![region("X", 0.0, 30.0)], Vec::new());
    run(&mut timer_loop, 1);
    let names: Vec<&str> = timer_loop
        .catalog()
        .songs
        .iter()
        .map(|song| song.name.as_str())
        .collect();
    assert_eq!(names, ["X"]);
    assert!(timer_loop.catalog().revision > first_revision);
    assert_eq!(timer_loop.rebuilds(), 1);
}

#[test]
fn switching_to_a_project_with_the_same_songs_still_starts_the_performance_over() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    timer_loop
        .port_mut()
        .switch_project(two_songs(), Vec::new());
    run(&mut timer_loop, 1);
    assert_eq!(timer_loop.phase(), Phase::Idle);
    assert_eq!(timer_loop.rebuilds(), 1);
}

#[test]
fn a_switch_during_a_performance_ends_it_and_says_so() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    run(&mut timer_loop, 10);
    assert_eq!(timer_loop.phase(), Phase::Playing);
    timer_loop
        .port_mut()
        .switch_project(vec![region("X", 0.0, 30.0)], Vec::new());
    run(&mut timer_loop, 1);
    assert_eq!(timer_loop.phase(), Phase::Idle);
    assert_eq!(timer_loop.port_mut().transport(), Transport::Stopped);
    assert!(
        timer_loop
            .take_events()
            .contains(&WireEvent::ProjectChanged)
    );
}

#[test]
fn the_new_project_gets_its_own_id_and_the_old_one_is_not_carried_over() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    let first = timer_loop.port_mut().ext_state("RC2", "project_id");
    assert!(first.is_some());
    timer_loop
        .port_mut()
        .switch_project(two_songs(), Vec::new());
    run(&mut timer_loop, 1);
    let second = timer_loop.port_mut().ext_state("RC2", "project_id");
    assert!(second.is_some());
    assert_ne!(first, second);
}

#[test]
fn staying_in_the_same_project_reports_no_switch() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    run(&mut timer_loop, 20);
    assert_eq!(timer_loop.rebuilds(), 0);
    assert!(
        !timer_loop
            .take_events()
            .contains(&WireEvent::ProjectChanged)
    );
}

#[test]
fn opening_another_project_in_the_same_tab_is_a_switch_too() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop.command(Input::Play);
    timer_loop
        .port_mut()
        .replace_regions(vec![region("X", 0.0, 30.0)]);
    timer_loop
        .port_mut()
        .set_ext_state("RC2", "project_id", "another-project");
    run(&mut timer_loop, 1);
    assert_eq!(timer_loop.phase(), Phase::Idle);
    assert!(
        timer_loop
            .take_events()
            .contains(&WireEvent::ProjectChanged)
    );
}

#[test]
fn an_edit_in_the_same_project_is_not_a_switch() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    timer_loop
        .port_mut()
        .replace_regions(vec![region("A", 0.0, 10.0)]);
    run(&mut timer_loop, 1);
    assert!(
        !timer_loop
            .take_events()
            .contains(&WireEvent::ProjectChanged)
    );
}

#[test]
fn a_copied_project_file_gets_its_own_id_and_a_moved_one_is_treated_the_same() {
    let mut fake = fake(two_songs(), Vec::new());
    fake.set_project_path(Some("/shows/a.RPP"));
    let mut timer_loop = TimerLoop::new(fake);
    let original = timer_loop.port_mut().ext_state("RC2", "project_id");
    assert!(original.is_some());
    assert_eq!(
        timer_loop
            .port_mut()
            .ext_state("RC2", "project_path")
            .as_deref(),
        Some("/shows/a.RPP")
    );
    timer_loop
        .port_mut()
        .set_project_path(Some("/shows/copy.RPP"));
    timer_loop.port_mut().replace_regions(two_songs());
    run(&mut timer_loop, 1);
    let copy = timer_loop.port_mut().ext_state("RC2", "project_id");
    assert!(copy.is_some());
    assert_ne!(copy, original);
    assert_eq!(
        timer_loop
            .port_mut()
            .ext_state("RC2", "project_path")
            .as_deref(),
        Some("/shows/copy.RPP")
    );
}

#[test]
fn an_unsaved_project_keeps_its_id_and_records_no_path() {
    let mut timer_loop = TimerLoop::new(fake(two_songs(), Vec::new()));
    let id = timer_loop.port_mut().ext_state("RC2", "project_id");
    timer_loop.port_mut().replace_regions(two_songs());
    run(&mut timer_loop, 1);
    assert_eq!(timer_loop.port_mut().ext_state("RC2", "project_id"), id);
    assert_eq!(timer_loop.port_mut().ext_state("RC2", "project_path"), None);
}
