use std::sync::atomic::AtomicUsize;

use link::LinkEvent;

use crate::fake::FakeFaultCheck;
use app_core::{EventBus, FakeClock};
use installer::FakeProcessCheck;

use super::*;

struct Rig {
    enabled: Arc<AtomicBool>,
    closed: Arc<AtomicUsize>,
    monitor: Arc<HealthMonitor>,
    watcher: CloseWithReaper,
}

fn rig(enabled: bool) -> Rig {
    let monitor = Arc::new(HealthMonitor::new(
        Arc::new(EventBus::default()),
        Arc::new(FakeClock::default()),
        Arc::new(FakeProcessCheck::default()),
        Arc::new(FakeFaultCheck::default()),
    ));
    let enabled = Arc::new(AtomicBool::new(enabled));
    let closed = Arc::new(AtomicUsize::new(0));
    let counter = closed.clone();
    let watcher = CloseWithReaper::new(
        enabled.clone(),
        monitor.clone(),
        Box::new(move || {
            counter.fetch_add(1, Ordering::Relaxed);
        }),
    );
    Rig {
        enabled,
        closed,
        monitor,
        watcher,
    }
}

fn connect(rig: &Rig) {
    rig.monitor.observe(&LinkEvent::Connected {
        extension_version: "1".into(),
    });
}

#[test]
fn it_closes_when_the_link_ends_after_reaper_said_it_is_quitting() {
    let rig = rig(true);
    connect(&rig);
    rig.monitor.observe(&LinkEvent::Closing);
    rig.watcher.observe(&LinkHealth::Lost);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 1);
}

#[test]
fn it_stays_open_when_the_link_ends_without_the_message() {
    let rig = rig(true);
    connect(&rig);
    rig.watcher.observe(&LinkHealth::Lost);
    rig.watcher
        .observe(&LinkHealth::Dead(LinkCause::ReaperNotRunning));
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
}

#[test]
fn it_stays_open_when_switched_off() {
    let rig = rig(false);
    connect(&rig);
    rig.monitor.observe(&LinkEvent::Closing);
    rig.watcher.observe(&LinkHealth::Lost);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
    rig.enabled.store(true, Ordering::Relaxed);
    rig.watcher.observe(&LinkHealth::Lost);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 1);
}

#[test]
fn a_new_connection_forgets_an_old_goodbye() {
    let rig = rig(true);
    connect(&rig);
    rig.monitor.observe(&LinkEvent::Closing);
    connect(&rig);
    rig.watcher.observe(&LinkHealth::Lost);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
}

#[test]
fn it_does_not_close_while_the_link_is_still_up() {
    let rig = rig(true);
    connect(&rig);
    rig.monitor.observe(&LinkEvent::Closing);
    rig.watcher.observe(&LinkHealth::Connected);
    rig.watcher.observe(&LinkHealth::Degraded);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
}
