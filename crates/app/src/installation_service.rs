//! What the first-run wizard asks of the app: look at the REAPER, install into it.

use std::sync::Arc;

use protocol::{InstallationView, WizardStepStatus};

use crate::extension_installer::ExtensionInstaller;
use crate::process_check::ProcessCheck;
use crate::reaper_install::ReaperInstall;
use crate::wizard_step_list::wizard_step_list;

/// Looks at one REAPER and installs the bundled extension into it.
pub struct InstallationService {
    // Boxed: with Windows paths the installer alone is over 100 bytes.
    installer: Box<ExtensionInstaller>,
    install: ReaperInstall,
    process_check: Arc<dyn ProcessCheck>,
}

impl InstallationService {
    /// A service for `install`.
    pub fn new(
        installer: ExtensionInstaller,
        install: ReaperInstall,
        process_check: Arc<dyn ProcessCheck>,
    ) -> Self {
        Self {
            installer: Box::new(installer),
            install,
            process_check,
        }
    }

    /// The wizard as it stands; `connected` is whether the extension has answered the handshake.
    #[must_use]
    pub fn view(&self, connected: bool) -> InstallationView {
        let report = self.installer.inspect(&self.install);
        let steps = wizard_step_list(&report, self.process_check.reaper_is_running(), connected);
        let can_install = steps.iter().any(|step| {
            step.id == protocol::WizardStepId::InstallExtension
                && step.status == WizardStepStatus::Current
        });
        InstallationView {
            steps,
            folder: self.install.resource_folder.display().to_string(),
            can_install,
            complete: connected,
        }
    }

    /// Installs the extension (the wizard's button and Settings' "Repair"), then looks again.
    pub fn install(&self, connected: bool) -> Result<InstallationView, String> {
        self.installer
            .install(&self.install)
            .map_err(|error| error.to_string())?;
        Ok(self.view(connected))
    }
}
