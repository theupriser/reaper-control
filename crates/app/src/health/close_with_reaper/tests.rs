use std::sync::atomic::AtomicUsize;

use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Rig {
    directory: PathBuf,
    enabled: Arc<AtomicBool>,
    closed: Arc<AtomicUsize>,
    watcher: CloseWithReaper,
}

fn rig(name: &str, enabled: bool) -> Result<Rig, std::io::Error> {
    let directory = std::env::temp_dir().join(format!("rc2-close-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory)?;
    let enabled = Arc::new(AtomicBool::new(enabled));
    let closed = Arc::new(AtomicUsize::new(0));
    let counter = closed.clone();
    let watcher = CloseWithReaper::new(
        enabled.clone(),
        directory.join("running"),
        Box::new(move || {
            counter.fetch_add(1, Ordering::Relaxed);
        }),
    );
    Ok(Rig {
        directory,
        enabled,
        closed,
        watcher,
    })
}

const GONE: LinkHealth = LinkHealth::Dead(LinkCause::ReaperNotRunning);

#[test]
fn it_closes_when_a_link_that_worked_ends_with_a_clean_exit() -> TestResult {
    let rig = rig("clean", true)?;
    std::fs::write(rig.directory.join("running"), "")?;
    rig.watcher.observe(&LinkHealth::Connected);
    std::fs::remove_file(rig.directory.join("running"))?;
    rig.watcher.observe(&LinkHealth::Lost);
    rig.watcher.observe(&GONE);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 1);
    std::fs::remove_dir_all(&rig.directory)?;
    Ok(())
}

#[test]
fn it_stays_open_after_a_crash() -> TestResult {
    let rig = rig("crash", true)?;
    std::fs::write(rig.directory.join("running"), "")?;
    rig.watcher.observe(&LinkHealth::Connected);
    rig.watcher.observe(&GONE);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
    std::fs::remove_dir_all(&rig.directory)?;
    Ok(())
}

#[test]
fn it_does_nothing_when_switched_off_or_never_connected() -> TestResult {
    let rig = rig("off", false)?;
    std::fs::write(rig.directory.join("running"), "")?;
    rig.watcher.observe(&LinkHealth::Connected);
    rig.watcher.observe(&GONE);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
    std::fs::remove_file(rig.directory.join("running"))?;
    rig.enabled.store(true, Ordering::Relaxed);
    let fresh = CloseWithReaper::new(
        rig.enabled.clone(),
        rig.directory.join("running"),
        Box::new(|| {}),
    );
    fresh.observe(&GONE);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
    std::fs::remove_dir_all(&rig.directory)?;
    Ok(())
}

#[test]
fn it_wants_to_have_seen_the_file_while_connected() -> TestResult {
    let rig = rig("unseen", true)?;
    rig.watcher.observe(&LinkHealth::Connected);
    rig.watcher.observe(&GONE);
    assert_eq!(rig.closed.load(Ordering::Relaxed), 0);
    std::fs::remove_dir_all(&rig.directory)?;
    Ok(())
}
