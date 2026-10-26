//! Updating an installed extension: ask REAPER to quit, replace the file, start REAPER again.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use installer::{
    ExtensionInstaller, FakeProcessCheck, FakeReaperControl, InstallPlatform, InstallationService,
    PlainExtensionPreparer, ProcessCheck, ReaperInstall, UpdateError,
};

const MACH_O_ARM64: [u8; 8] = [0xcf, 0xfa, 0xed, 0xfe, 0x0c, 0x00, 0x00, 0x01];

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Setup {
    root: PathBuf,
    installed: PathBuf,
    process: Arc<FakeProcessCheck>,
    control: Arc<FakeReaperControl>,
    service: InstallationService,
}

impl Drop for Setup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// An old extension installed, a newer one bundled, REAPER running.
fn setup() -> Result<Setup, std::io::Error> {
    let root = std::env::temp_dir().join(format!(
        "rc2-update-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    let plugins = root.join("resource").join("UserPlugins");
    fs::create_dir_all(&plugins)?;
    let installed = plugins.join(InstallPlatform::MacArm64.library_file_name());
    fs::write(&installed, b"old extension")?;
    let executable = root.join("REAPER");
    fs::write(&executable, MACH_O_ARM64)?;
    let bundled = root.join("bundled.dylib");
    fs::write(&bundled, b"new extension")?;
    let process = Arc::new(FakeProcessCheck::default());
    process.set_running(true);
    let control = Arc::new(FakeReaperControl::new(process.clone()));
    let installer = ExtensionInstaller::new(
        InstallPlatform::MacArm64,
        bundled,
        process.clone(),
        Box::new(PlainExtensionPreparer),
    );
    let install = ReaperInstall {
        resource_folder: root.join("resource"),
        executable: Some(executable),
    };
    let service = InstallationService::new(installer, install, process.clone(), control.clone())
        .with_close_timeout(Duration::from_millis(20));
    Ok(Setup {
        root,
        installed,
        process,
        control,
        service,
    })
}

#[test]
fn an_older_extension_is_offered_as_an_update() -> Result<(), std::io::Error> {
    let setup = setup()?;
    let view = setup.service.view(true);
    assert!(view.update_available && view.reaper_running);
    Ok(())
}

#[test]
fn the_update_quits_reaper_replaces_the_file_and_starts_reaper_again()
-> Result<(), Box<dyn std::error::Error>> {
    let setup = setup()?;
    let view = setup.service.update(true, false, true)?;
    assert_eq!(fs::read(&setup.installed)?, b"new extension");
    assert_eq!(setup.control.calls(), ["quit", "launch"]);
    assert!(setup.process.reaper_is_running());
    assert!(!view.update_available);
    Ok(())
}

#[test]
fn without_restart_reaper_stays_closed() -> Result<(), Box<dyn std::error::Error>> {
    let setup = setup()?;
    setup.service.update(true, false, false)?;
    assert_eq!(setup.control.calls(), ["quit"]);
    assert!(!setup.process.reaper_is_running());
    Ok(())
}

#[test]
fn a_running_performance_is_never_interrupted() -> Result<(), Box<dyn std::error::Error>> {
    let setup = setup()?;
    let refused = setup.service.update(true, true, true);
    assert!(matches!(refused, Err(UpdateError::PerformanceRunning)));
    assert!(setup.control.calls().is_empty());
    assert_eq!(fs::read(&setup.installed)?, b"old extension");
    Ok(())
}

#[test]
fn a_reaper_that_stays_open_keeps_the_old_extension_and_is_not_killed()
-> Result<(), Box<dyn std::error::Error>> {
    let setup = setup()?;
    setup.control.set_closes_when_asked(false);
    let refused = setup.service.update(true, false, true);
    assert!(matches!(refused, Err(UpdateError::ReaperStayedOpen(_))));
    assert_eq!(fs::read(&setup.installed)?, b"old extension");
    assert_eq!(setup.control.calls(), ["quit"]);
    Ok(())
}

#[test]
fn a_current_extension_has_nothing_to_update() -> Result<(), Box<dyn std::error::Error>> {
    let setup = setup()?;
    setup.service.update(true, false, false)?;
    let again = setup.service.update(true, false, false);
    assert!(matches!(again, Err(UpdateError::NothingToUpdate)));
    Ok(())
}
