//! Builds the installation service for the REAPER this app was pointed at.

use std::path::PathBuf;
use std::sync::Arc;

use crate::bundled_extension::bundled_extension;
use crate::extension_installer::ExtensionInstaller;
use crate::extension_preparer::ExtensionPreparer;
use crate::install_platform::InstallPlatform;
use crate::installation_service::InstallationService;
use crate::mac_extension_preparer::MacExtensionPreparer;
use crate::plain_extension_preparer::PlainExtensionPreparer;
use crate::process_check::ProcessCheck;
use crate::reaper_folder_argument::reaper_folder_argument;
use crate::reaper_install::ReaperInstall;

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
