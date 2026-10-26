//! Builds the installation service for the REAPER this app was pointed at.

use std::path::PathBuf;
use std::sync::Arc;

use installer::ExtensionInstaller;
use installer::ExtensionPreparer;
use installer::InstallPlatform;
use installer::InstallationService;
use installer::MacExtensionPreparer;
use installer::PlainExtensionPreparer;
use installer::ProcessCheck;
use installer::ReaperInstall;
use installer::SystemReaperControl;
use installer::bundled_extension;
use installer::reaper_folder_argument;

/// The service for the REAPER named by `--reaper-folder`, else the normal one; `None` on a system
/// the extension is not built for.
#[must_use]
pub fn start_installation(processes: Arc<dyn ProcessCheck>) -> Option<InstallationService> {
    let platform = InstallPlatform::current()?;
    let user_folder = user_folder(platform)?;
    let chosen_folder = reaper_folder_argument(std::env::args().skip(1));
    let control = SystemReaperControl::new(chosen_folder.is_some());
    let install = ReaperInstall::locate(platform, chosen_folder, &user_folder);
    let preparer: Box<dyn ExtensionPreparer> = match platform {
        InstallPlatform::MacArm64 => Box::new(MacExtensionPreparer),
        InstallPlatform::WindowsX64 => Box::new(PlainExtensionPreparer),
    };
    let installer = ExtensionInstaller::new(
        platform,
        bundled_extension(platform),
        processes.clone(),
        preparer,
    );
    Some(InstallationService::new(
        installer,
        install,
        processes,
        Arc::new(control),
    ))
}

fn user_folder(platform: InstallPlatform) -> Option<PathBuf> {
    let variable = match platform {
        InstallPlatform::MacArm64 => "HOME",
        InstallPlatform::WindowsX64 => "APPDATA",
    };
    std::env::var_os(variable).map(PathBuf::from)
}
