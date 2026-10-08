use std::sync::Mutex;

use crate::fake_clock::FakeClock;
use crate::fake_fault_check::FakeFaultCheck;
use crate::fake_process_check::FakeProcessCheck;

use super::*;

struct Setup {
    monitor: HealthMonitor,
    clock: Arc<FakeClock>,
    processes: Arc<FakeProcessCheck>,
    faults: Arc<FakeFaultCheck>,
    heard: Arc<Mutex<Vec<LinkHealth>>>,
}

fn setup() -> Setup {
    let events = Arc::new(EventBus::default());
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&heard);
    events.subscribe(move |event| {
        if let (AppEvent::LinkHealthChanged { health }, Ok(mut list)) = (event, sink.lock()) {
            list.push(health.clone());
        }
    });
    let clock = Arc::new(FakeClock::default());
    let processes = Arc::new(FakeProcessCheck::default());
    let faults = Arc::new(FakeFaultCheck::default());
    Setup {
        monitor: HealthMonitor::new(events, clock.clone(), processes.clone(), faults.clone()),
        clock,
        processes,
        faults,
        heard,
    }
}

fn connected() -> LinkEvent {
    LinkEvent::Connected {
        extension_version: "1".into(),
    }
}

fn announced(setup: &Setup) -> Vec<LinkHealth> {
    setup
        .heard
        .lock()
        .map(|list| list.clone())
        .unwrap_or_default()
}

#[test]
fn it_starts_lost_and_gets_better_and_worse_step_by_step() {
    let setup = setup();
    assert_eq!(setup.monitor.health(), LinkHealth::Lost);
    setup.monitor.observe(&connected());
    setup.monitor.observe(&LinkEvent::Quiet);
    setup.monitor.observe(&LinkEvent::Recovered);
    setup.monitor.observe(&LinkEvent::Quiet);
    setup.monitor.observe(&LinkEvent::Disconnected);
    assert_eq!(
        announced(&setup),
        vec![
            LinkHealth::Connected,
            LinkHealth::Degraded,
            LinkHealth::Connected,
            LinkHealth::Degraded,
            LinkHealth::Lost
        ]
    );
}

#[test]
fn a_repeat_is_not_news() {
    let setup = setup();
    setup.monitor.observe(&connected());
    setup.monitor.observe(&connected());
    setup.monitor.observe(&LinkEvent::Disconnected);
    setup.monitor.observe(&LinkEvent::Disconnected);
    assert_eq!(
        announced(&setup),
        vec![LinkHealth::Connected, LinkHealth::Lost]
    );
}

#[test]
fn a_lost_link_is_named_only_after_the_grace_period() {
    let setup = setup();
    setup.monitor.observe(&connected());
    setup.monitor.observe(&LinkEvent::Disconnected);
    setup.clock.advance(Duration::from_secs(4));
    setup.monitor.tick();
    assert_eq!(setup.monitor.health(), LinkHealth::Lost);
    setup.clock.advance(Duration::from_secs(1));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ReaperNotRunning)
    );
}

#[test]
fn the_cause_follows_whether_reaper_runs() {
    let setup = setup();
    setup.clock.advance(Duration::from_secs(5));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ReaperNotRunning)
    );
    setup.processes.set_running(true);
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ReaperNotRunning),
        "asked again too soon"
    );
    setup.clock.advance(Duration::from_secs(5));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ExtensionNotLoaded)
    );
    setup.monitor.tick();
    assert_eq!(announced(&setup).len(), 2);
}

#[test]
fn another_protocol_is_dead_at_once_and_a_connection_ends_it() {
    let setup = setup();
    setup.monitor.observe(&LinkEvent::Outdated { found: 0 });
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ExtensionOutdated { found: 0 })
    );
    setup.processes.set_running(true);
    setup.clock.advance(Duration::from_secs(60));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ExtensionOutdated { found: 0 })
    );
    setup.monitor.observe(&connected());
    assert_eq!(setup.monitor.health(), LinkHealth::Connected);
}

#[test]
fn an_outdated_extension_is_replaced_by_reaper_not_running_when_reaper_quits() {
    let setup = setup();
    setup.monitor.observe(&LinkEvent::Outdated { found: 0 });
    setup.clock.advance(Duration::from_secs(60));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ReaperNotRunning)
    );
}

#[test]
fn a_running_reaper_with_a_faulted_extension_names_the_fault() {
    let setup = setup();
    setup.processes.set_running(true);
    setup.faults.set_reason(Some("safe mode"));
    setup.clock.advance(Duration::from_secs(5));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ExtensionFaulted {
            reason: "safe mode".into()
        })
    );
    setup.faults.set_reason(None);
    setup.clock.advance(Duration::from_secs(5));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ExtensionNotLoaded),
        "the fault was cleared"
    );
}

#[test]
fn a_fault_left_behind_is_ignored_while_reaper_is_not_running() {
    let setup = setup();
    setup.faults.set_reason(Some("safe mode"));
    setup.clock.advance(Duration::from_secs(5));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ReaperNotRunning)
    );
}

#[test]
fn an_extension_that_reports_a_fault_while_still_answering_is_dead() {
    let setup = setup();
    setup.monitor.observe(&connected());
    setup.clock.advance(Duration::from_secs(5));
    setup.monitor.tick();
    assert_eq!(setup.monitor.health(), LinkHealth::Connected);
    setup.faults.set_reason(Some("crashed"));
    setup.clock.advance(Duration::from_secs(5));
    setup.monitor.tick();
    assert_eq!(
        setup.monitor.health(),
        LinkHealth::Dead(LinkCause::ExtensionFaulted {
            reason: "crashed".into()
        })
    );
}
