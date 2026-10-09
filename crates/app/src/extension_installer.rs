//! Puts the extension into a REAPER, checks it, repairs it and takes it out again (SPEC §13).
//! Nothing here edits a REAPER settings file, and the app never starts or stops REAPER.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::binary_architecture::BinaryArchitecture;
use crate::extension_preparer::ExtensionPreparer;
use crate::install_action::InstallAction;
use crate::install_error::InstallError;
use crate::install_item::InstallItem;
use crate::install_item_kind::InstallItemKind;
use crate::install_plan::InstallPlan;
use crate::install_platform::InstallPlatform;
use crate::install_report::InstallReport;
use crate::install_status::InstallStatus;
use crate::process_check::ProcessCheck;
use crate::reaper_install::ReaperInstall;

const BACKUP_FOLDER: &str = "RC2-backups";
const PLUGIN_FOLDER: &str = "UserPlugins";

/// Installs the bundled extension into a REAPER and keeps it healthy.
pub struct ExtensionInstaller {
    platform: InstallPlatform,
    /// The extension file that ships with the app.
    bundled: PathBuf,
    process_check: Arc<dyn ProcessCheck>,
    preparer: Box<dyn ExtensionPreparer>,
}

impl ExtensionInstaller {
    /// An installer for `platform` that copies `bundled`.
    pub fn new(
        platform: InstallPlatform,
        bundled: PathBuf,
        process_check: Arc<dyn ProcessCheck>,
        preparer: Box<dyn ExtensionPreparer>,
    ) -> Self {
        Self {
            platform,
            bundled,
            process_check,
            preparer,
        }
    }

    /// Where the extension lives, or would live, in `install`.
    #[must_use]
    pub fn installed_path(&self, install: &ReaperInstall) -> PathBuf {
        install
            .resource_folder
            .join(PLUGIN_FOLDER)
            .join(self.platform.library_file_name())
    }

    /// Where the previous version is kept.
    #[must_use]
    pub fn backup_path(&self, install: &ReaperInstall) -> PathBuf {
        install
            .resource_folder
            .join(BACKUP_FOLDER)
            .join(self.platform.library_file_name())
    }

    /// Looks at `install` and reports each item; changes nothing.
    #[must_use]
    pub fn inspect(&self, install: &ReaperInstall) -> InstallReport {
        let extension = self.extension_item(install);
        let needs_change = extension.status != InstallStatus::Ok;
        InstallReport {
            items: vec![
                Self::resource_folder_item(install),
                Self::writable_item(install),
                self.running_item(needs_change && self.installed_path(install).exists()),
                self.architecture_item(install),
                extension,
            ],
        }
    }

    /// What `install` would do, without doing it (the dry run).
    pub fn plan(&self, install: &ReaperInstall) -> Result<InstallPlan, InstallError> {
        if !self.bundled.is_file() {
            return Err(InstallError::BundleMissing {
                path: self.bundled.clone(),
            });
        }
        if let Some(found) = self.architecture_of(install) {
            let expected = self.platform.expected_architecture();
            if found != expected {
                return Err(InstallError::WrongArchitecture { found, expected });
            }
        }
        let target = self.installed_path(install);
        if same_content(&self.bundled, &target) {
            return Ok(InstallPlan::default());
        }
        let mut actions = Vec::new();
        if target.exists() {
            if self.process_check.reaper_is_running() {
                return Err(InstallError::ReaperIsRunning);
            }
            actions.push(InstallAction::KeepPrevious {
                from: target.clone(),
                to: self.backup_path(install),
            });
        }
        actions.push(InstallAction::CopyExtension {
            from: self.bundled.clone(),
            to: target.clone(),
        });
        actions.push(InstallAction::PrepareLibrary { library: target });
        Ok(InstallPlan { actions })
    }

    /// Carries out `plan`. If the new file cannot be prepared, the old state is put back.
    pub fn apply(&self, plan: &InstallPlan) -> Result<(), InstallError> {
        let mut kept: Option<(PathBuf, PathBuf)> = None;
        for action in &plan.actions {
            match action {
                InstallAction::KeepPrevious { from, to } => {
                    copy_file(from, to)?;
                    kept = Some((from.clone(), to.clone()));
                }
                InstallAction::CopyExtension { from, to } => copy_into_place(from, to)?,
                InstallAction::PrepareLibrary { library } => {
                    if let Err(reason) = self.preparer.prepare(library) {
                        match &kept {
                            Some((original, backup)) => copy_file(backup, original)?,
                            None => remove_file(library)?,
                        }
                        return Err(InstallError::Preparation(reason));
                    }
                }
            }
        }
        Ok(())
    }

    /// Installs or updates the extension; returns what was done (empty when it was current).
    pub fn install(&self, install: &ReaperInstall) -> Result<InstallPlan, InstallError> {
        let plan = self.plan(install)?;
        self.apply(&plan)?;
        Ok(plan)
    }

    /// Fixes whatever `inspect` finds fixable: the same as installing, spelled for the Settings
    /// button.
    pub fn repair(&self, install: &ReaperInstall) -> Result<InstallPlan, InstallError> {
        self.install(install)
    }

    /// Removes the extension; with `restore_previous` the kept older version comes back.
    pub fn uninstall(
        &self,
        install: &ReaperInstall,
        restore_previous: bool,
    ) -> Result<(), InstallError> {
        let target = self.installed_path(install);
        if !target.exists() {
            return Err(InstallError::NotInstalled);
        }
        if self.process_check.reaper_is_running() {
            return Err(InstallError::ReaperIsRunning);
        }
        remove_file(&target)?;
        let backup = self.backup_path(install);
        if restore_previous && backup.exists() {
            copy_file(&backup, &target)?;
        }
        Ok(())
    }

    fn architecture_of(&self, install: &ReaperInstall) -> Option<BinaryArchitecture> {
        let executable = install.executable.as_ref()?;
        BinaryArchitecture::read(executable).ok().flatten()
    }

    fn resource_folder_item(install: &ReaperInstall) -> InstallItem {
        if install.resource_folder.is_dir() {
            InstallItem::new(InstallItemKind::ResourceFolder, InstallStatus::Ok, "found")
        } else {
            InstallItem::new(
                InstallItemKind::ResourceFolder,
                InstallStatus::Manual,
                "REAPER's folder was not found; start REAPER once, or choose its folder",
            )
        }
    }

    fn writable_item(install: &ReaperInstall) -> InstallItem {
        let writable = fs::metadata(&install.resource_folder)
            .map(|metadata| !metadata.permissions().readonly())
            .unwrap_or(true);
        if writable {
            InstallItem::new(
                InstallItemKind::Writable,
                InstallStatus::Ok,
                "can be written",
            )
        } else {
            InstallItem::new(
                InstallItemKind::Writable,
                InstallStatus::Manual,
                "the folder is read-only; change its permissions or run the app as the owner of the folder",
            )
        }
    }

    fn running_item(&self, blocks_change: bool) -> InstallItem {
        if blocks_change && self.process_check.reaper_is_running() {
            InstallItem::new(
                InstallItemKind::ReaperRunning,
                InstallStatus::Manual,
                "REAPER is running; close it to update the extension, then check again",
            )
        } else {
            InstallItem::new(
                InstallItemKind::ReaperRunning,
                InstallStatus::Ok,
                "no conflict",
            )
        }
    }

    fn architecture_item(&self, install: &ReaperInstall) -> InstallItem {
        let expected = self.platform.expected_architecture();
        match self.architecture_of(install) {
            Some(found) if found == expected => {
                InstallItem::new(InstallItemKind::Architecture, InstallStatus::Ok, "matches")
            }
            Some(found) => InstallItem::new(
                InstallItemKind::Architecture,
                InstallStatus::Manual,
                format!(
                    "this REAPER is built for {found:?}; install the {expected:?} version of REAPER"
                ),
            ),
            None => InstallItem::new(
                InstallItemKind::Architecture,
                InstallStatus::Unknown,
                "the REAPER program could not be read",
            ),
        }
    }

    fn extension_item(&self, install: &ReaperInstall) -> InstallItem {
        let target = self.installed_path(install);
        if !self.bundled.is_file() {
            InstallItem::new(
                InstallItemKind::Extension,
                InstallStatus::Manual,
                "the extension that ships with the app is missing; reinstall the app",
            )
        } else if same_content(&self.bundled, &target) {
            InstallItem::new(
                InstallItemKind::Extension,
                InstallStatus::Ok,
                "installed and current",
            )
        } else if target.exists() {
            InstallItem::new(
                InstallItemKind::Extension,
                InstallStatus::Fixable,
                "installed, but not the version that ships with the app",
            )
        } else {
            InstallItem::new(
                InstallItemKind::Extension,
                InstallStatus::Fixable,
                "not installed",
            )
        }
    }
}

fn same_content(first: &Path, second: &Path) -> bool {
    match (fs::read(first), fs::read(second)) {
        (Ok(first), Ok(second)) => first == second,
        _ => false,
    }
}

fn copy_file(from: &Path, to: &Path) -> Result<(), InstallError> {
    if let Some(folder) = to.parent() {
        fs::create_dir_all(folder).map_err(|source| io_error("create", folder, source))?;
    }
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|source| io_error("copy to", to, source))
}

/// Copies next to the target and renames, so REAPER never sees a half-written file.
fn copy_into_place(from: &Path, to: &Path) -> Result<(), InstallError> {
    let partial = to.with_extension("partial");
    copy_file(from, &partial)?;
    fs::rename(&partial, to).map_err(|source| io_error("replace", to, source))
}

fn remove_file(path: &Path) -> Result<(), InstallError> {
    fs::remove_file(path).map_err(|source| io_error("remove", path, source))
}

fn io_error(action: &'static str, path: &Path, source: std::io::Error) -> InstallError {
    InstallError::Io {
        action,
        path: path.to_path_buf(),
        source,
    }
}
