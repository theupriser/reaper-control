//! The first-run wizard's steps against fake REAPER folders in a temporary directory (WP 6.6).

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use app::{
    ExtensionInstaller, FakeProcessCheck, InstallPlatform, InstallationService,
    PlainExtensionPreparer, ReaperInstall,
};
use protocol::{InstallationView, WizardStepId, WizardStepStatus};

const MACH_O_ARM64: [u8; 8] = [0xcf, 0xfa, 0xed, 0xfe, 0x0c, 0x00, 0x00, 0x01];
const MACH_O_X86_64: [u8; 8] = [0xcf, 0xfa, 0xed, 0xfe, 0x07, 0x00, 0x00, 0x01];

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Wizard {
    root: PathBuf,
    process: Arc<FakeProcessCheck>,
    service: InstallationService,
}

impl Drop for Wizard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn wizard(reaper_header: &[u8], resource_exists: bool) -> Result<Wizard, std::io::Error> {
    let root = std::env::temp_dir().join(format!(
        "rc2-wizard-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    let resource = root.join("resource");
    if resource_exists {
        fs::create_dir_all(&resource)?;
    } else {
        fs::create_dir_all(&root)?;
    }
    let executable = root.join("REAPER");
    fs::write(&executable, reaper_header)?;
    let bundled = root.join("bundled.dylib");
    fs::write(&bundled, b"extension")?;
    let process = Arc::new(FakeProcessCheck::default());
    let installer = ExtensionInstaller::new(
        InstallPlatform::MacArm64,
        bundled,
        process.clone(),
        Box::new(PlainExtensionPreparer),
    );
    let install = ReaperInstall {
        resource_folder: resource,
        executable: Some(executable),
    };
    let service = InstallationService::new(installer, install, process.clone());
    Ok(Wizard {
        root,
        process,
        service,
    })
}

fn statuses(view: &InstallationView) -> Vec<(WizardStepId, WizardStepStatus)> {
    view.steps
        .iter()
        .map(|step| (step.id, step.status))
        .collect()
}

use WizardStepId::{Connect, FindReaper, InstallExtension, RestartReaper};
use WizardStepStatus::{Current, Done, NeedsYou, Waiting};

#[test]
fn a_fresh_reaper_starts_at_install_and_the_wizard_walks_to_connected() -> Result<(), std::io::Error>
{
    let wizard = wizard(&MACH_O_ARM64, true)?;
    let first = wizard.service.view(false);
    assert_eq!(
        statuses(&first),
        [
            (FindReaper, Done),
            (InstallExtension, Current),
            (RestartReaper, Waiting),
            (Connect, Waiting)
        ]
    );
    assert!(first.can_install && !first.complete);

    let installed = wizard
        .service
        .install(false)
        .map_err(std::io::Error::other)?;
    assert_eq!(
        statuses(&installed),
        [
            (FindReaper, Done),
            (InstallExtension, Done),
            (RestartReaper, Current),
            (Connect, Waiting)
        ]
    );
    assert!(!installed.can_install);
    assert!(installed.steps[2].advice.contains("Open REAPER"));

    wizard.process.set_running(true);
    assert!(
        wizard.service.view(false).steps[2]
            .advice
            .contains("Quit REAPER")
    );

    let connected = wizard.service.view(true);
    assert!(
        statuses(&connected)
            .iter()
            .all(|(_, status)| *status == Done)
    );
    assert!(connected.complete);
    Ok(())
}

#[test]
fn a_missing_resource_folder_needs_the_person_and_nothing_else_moves() -> Result<(), std::io::Error>
{
    let wizard = wizard(&MACH_O_ARM64, false)?;
    let view = wizard.service.view(false);
    assert_eq!(
        statuses(&view),
        [
            (FindReaper, NeedsYou),
            (InstallExtension, Waiting),
            (RestartReaper, Waiting),
            (Connect, Waiting)
        ]
    );
    assert!(
        view.steps[0]
            .advice
            .contains("REAPER's folder was not found")
    );
    assert!(!view.can_install);
    Ok(())
}

#[test]
fn an_intel_reaper_is_refused_with_its_reason() -> Result<(), std::io::Error> {
    let wizard = wizard(&MACH_O_X86_64, true)?;
    let view = wizard.service.view(false);
    assert_eq!(view.steps[0].status, NeedsYou);
    assert!(!view.can_install);
    let refused = wizard.service.install(false);
    assert!(refused.is_err_and(|reason| reason.contains("built for")));
    Ok(())
}
