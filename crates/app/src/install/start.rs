//! Builds the installation service for the REAPER this app was pointed at.

use std::path::PathBuf;
use std::sync::Arc;

use crate::health::ProcessCheck;
use crate::install::ExtensionInstaller;
use crate::install::ExtensionPreparer;
use crate::install::InstallPlatform;
use crate::install::InstallationService;
use crate::install::MacExtensionPreparer;
use crate::install::PlainExtensionPreparer;
use crate::install::ReaperInstall;
use crate::install::bundled_extension;
use crate::install::reaper_folder_argument;

/// The service for the REAPER named by `--reaper-folder`, else the normal one; `None` on a system
/// the extension is not built for.
#[must_use]
pub fn start_installation(processes: Arc<dyn ProcessCheck>) -> Option<InstallationService> {
    let platform = InstallPlatform::current()?;
    let user_folder = user_folder(platform)?;
    let install = ReaperInstall::locate(
        platform,
        reaper_folder_argument(std::env::args().skip(1)),
        &user_folder,
    );
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
    Some(InstallationService::new(installer, install, processes))
}

fn user_folder(platform: InstallPlatform) -> Option<PathBuf> {
    let variable = match platform {
        InstallPlatform::MacArm64 => "HOME",
        InstallPlatform::WindowsX64 => "APPDATA",
    };
    std::env::var_os(variable).map(PathBuf::from)
}
