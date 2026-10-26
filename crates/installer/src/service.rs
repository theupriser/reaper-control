//! What the first-run wizard asks of the app: look at the REAPER, install into it, update it.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use protocol::{InstallationView, WizardStepStatus};

use crate::ExtensionInstaller;
use crate::ProcessCheck;
use crate::ReaperControl;
use crate::ReaperInstall;
use crate::UpdateError;
use crate::wizard_step_list;

const CLOSE_POLL: Duration = Duration::from_millis(250);

/// Looks at one REAPER and installs the bundled extension into it.
pub struct InstallationService {
    // Boxed: with Windows paths the installer alone is over 100 bytes.
    installer: Box<ExtensionInstaller>,
    install: ReaperInstall,
    process_check: Arc<dyn ProcessCheck>,
    control: Arc<dyn ReaperControl>,
    /// How long REAPER gets to close after the request; a save prompt may be waiting for the person.
    close_timeout: Duration,
}

impl InstallationService {
    /// A service for `install`.
    pub fn new(
        installer: ExtensionInstaller,
        install: ReaperInstall,
        process_check: Arc<dyn ProcessCheck>,
        control: Arc<dyn ReaperControl>,
    ) -> Self {
        Self {
            installer: Box::new(installer),
            install,
            process_check,
            control,
            close_timeout: Duration::from_secs(60),
        }
    }

    /// How long an update waits for REAPER to close.
    #[must_use]
    pub fn with_close_timeout(mut self, close_timeout: Duration) -> Self {
        self.close_timeout = close_timeout;
        self
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
        let reaper_running = self.process_check.reaper_is_running();
        let update_available = can_install && self.installer.installed_path(&self.install).exists();
        InstallationView {
            steps,
            folder: self.install.resource_folder.display().to_string(),
            can_install,
            complete: connected,
            update_available,
            reaper_running,
        }
    }

    /// Installs the extension (the wizard's button and Settings' "Repair"), then looks again.
    pub fn install(&self, connected: bool) -> Result<InstallationView, String> {
        self.installer
            .install(&self.install)
            .map_err(|error| error.to_string())?;
        Ok(self.view(connected))
    }

    /// Updates the installed extension: asks REAPER to quit (it may ask the person to save, and is
    /// never killed), puts the new file in place, and starts REAPER again when `restart` is set.
    /// `performing` is whether a show is running; then nothing is touched.
    pub fn update(
        &self,
        connected: bool,
        performing: bool,
        restart: bool,
    ) -> Result<InstallationView, UpdateError> {
        if performing {
            return Err(UpdateError::PerformanceRunning);
        }
        if !self.view(connected).update_available {
            return Err(UpdateError::NothingToUpdate);
        }
        if self.process_check.reaper_is_running() {
            self.control.quit().map_err(UpdateError::ReaperStayedOpen)?;
            self.wait_until_closed()?;
        }
        self.installer.install(&self.install)?;
        if restart {
            self.control
                .launch(&self.install)
                .map_err(UpdateError::RestartFailed)?;
        }
        Ok(self.view(false))
    }

    fn wait_until_closed(&self) -> Result<(), UpdateError> {
        let started = Instant::now();
        while self.process_check.reaper_is_running() {
            if started.elapsed() >= self.close_timeout {
                return Err(UpdateError::ReaperStayedOpen(
                    "still running after the request".to_owned(),
                ));
            }
            thread::sleep(CLOSE_POLL.min(self.close_timeout));
        }
        Ok(())
    }
}
