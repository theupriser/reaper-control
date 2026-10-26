//! Runs the extension update against a real, isolated REAPER (testing/scripts/update_live_check.sh).
//! usage: update_live <REAPER program> <REAPER resource folder> <extension to put in place>
//!
//! The update quits REAPER by its program name, which would also close a REAPER you are using, so the
//! script refuses to start while another REAPER runs. The restart is done with the isolated folder's
//! `reaper.ini`, as the app does for a REAPER chosen with `--reaper-folder`.

use std::sync::Arc;
use std::time::Duration;

use installer::{
    ExtensionInstaller, ExtensionPreparer, InstallPlatform, InstallationService,
    MacExtensionPreparer, PlainExtensionPreparer, ProcessCheck, ReaperControl, ReaperInstall,
    SystemProcessCheck, SystemReaperControl,
};

/// Quits like the normal REAPER (the only one running), starts like a chosen folder.
struct IsolatedControl {
    quitting: SystemReaperControl,
    starting: SystemReaperControl,
}

impl ReaperControl for IsolatedControl {
    fn quit(&self) -> Result<(), String> {
        self.quitting.quit()
    }

    fn launch(&self, install: &ReaperInstall) -> Result<(), String> {
        self.starting.launch(install)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [executable, resource_folder, extension] = arguments.as_slice() else {
        eprintln!("usage: update_live <REAPER program> <REAPER resource folder> <extension>");
        std::process::exit(2);
    };
    let platform = InstallPlatform::current().ok_or("this system is not supported")?;
    let processes: Arc<dyn ProcessCheck> = Arc::new(SystemProcessCheck);
    let preparer: Box<dyn ExtensionPreparer> = match platform {
        InstallPlatform::MacArm64 => Box::new(MacExtensionPreparer),
        InstallPlatform::WindowsX64 => Box::new(PlainExtensionPreparer),
    };
    let installer =
        ExtensionInstaller::new(platform, extension.into(), processes.clone(), preparer);
    let install = ReaperInstall {
        resource_folder: resource_folder.into(),
        executable: Some(executable.into()),
    };
    let control = Arc::new(IsolatedControl {
        quitting: SystemReaperControl::new(false),
        starting: SystemReaperControl::new(true),
    });
    let service = InstallationService::new(installer, install, processes, control)
        .with_close_timeout(Duration::from_secs(60));

    let before = service.view(true);
    println!(
        "before: update available {}, REAPER running {}",
        before.update_available, before.reaper_running
    );
    let after = service.update(true, false, true)?;
    println!(
        "after: update available {}, REAPER running {}",
        after.update_available, after.reaper_running
    );
    Ok(())
}
