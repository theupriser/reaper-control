//! Bringing setlists into the project: from the backup copy, or from v1's files.

use std::path::PathBuf;
use std::sync::Arc;

use protocol::{Command, ImportOffer, LinkView, SetlistInfo, SetlistTransferView};

use crate::command_bus::CommandBus;
use crate::import_error::ImportError;
use crate::legacy_file::LegacyFile;
use crate::legacy_resolver::resolve;
use crate::mirror_repository::MirrorRepository;

/// Offers and carries out a restore or an import. Both end in `SaveSetlist` through the
/// command bus, so the project is only ever changed the way an edit changes it.
pub struct SetlistTransfer {
    mirror: Arc<dyn MirrorRepository>,
    legacy_directory: Option<PathBuf>,
    bus: Arc<CommandBus>,
}

impl SetlistTransfer {
    /// A service over the backup copy and the folder of v1's setlist files, if v1 left one.
    #[must_use]
    pub fn new(
        mirror: Arc<dyn MirrorRepository>,
        legacy_directory: Option<PathBuf>,
        bus: Arc<CommandBus>,
    ) -> Self {
        Self {
            mirror,
            legacy_directory,
            bus,
        }
    }

    /// What can be moved into the project that `link` shows.
    #[must_use]
    pub fn view(&self, link: &LinkView) -> SetlistTransferView {
        let project = &link.catalog.project_id;
        if project.is_empty() {
            return SetlistTransferView {
                problem: Some("The project is not known yet: connect to REAPER first.".into()),
                ..SetlistTransferView::default()
            };
        }
        let mut view = SetlistTransferView::default();
        match self.missing_from_backup(link) {
            Ok(missing) => view.restorable = missing.into_iter().map(|s| s.name).collect(),
            Err(error) => view.problem = Some(error),
        }
        match self.offers(link) {
            Ok(offers) => view.imports = offers,
            Err(error) => view.problem = Some(error),
        }
        view
    }

    /// Puts the setlists of the backup copy that the project lacks back into it.
    ///
    /// # Errors
    /// The reason, as text for the screen, when the copy cannot be read or a command is refused.
    pub fn restore(&self, link: &LinkView) -> Result<usize, String> {
        let missing = self.missing_from_backup(link)?;
        for setlist in &missing {
            self.save(setlist.id.clone(), setlist.name.clone(), setlist)?;
        }
        Ok(missing.len())
    }

    /// Brings the chosen v1 setlists into the project; songs it does not have are left out.
    ///
    /// # Errors
    /// The reason, as text for the screen, when the file cannot be read or a command is refused.
    pub fn import(&self, link: &LinkView, ids: &[String]) -> Result<usize, String> {
        let mut done = 0;
        for legacy in self.legacy_setlists(link)?.setlists {
            let present = link.catalog.setlists.iter().any(|s| s.id == legacy.id);
            if present || !ids.contains(&legacy.id) {
                continue;
            }
            let resolved = resolve(&legacy, &link.catalog.project_songs);
            self.save(legacy.id, legacy.name, &resolved.setlist)?;
            done += 1;
        }
        Ok(done)
    }

    fn save(&self, id: String, name: String, setlist: &SetlistInfo) -> Result<(), String> {
        self.bus
            .dispatch(Command::SaveSetlist {
                id,
                name,
                entries: setlist.entries.clone(),
                expected_revision: 0,
            })
            .map_err(|error| error.to_string())
    }

    fn missing_from_backup(&self, link: &LinkView) -> Result<Vec<SetlistInfo>, String> {
        let copy = self
            .mirror
            .restore(&link.catalog.project_id)
            .map_err(|error| error.to_string())?;
        Ok(copy
            .into_iter()
            .filter(|s| !link.catalog.setlists.iter().any(|have| have.id == s.id))
            .collect())
    }

    fn offers(&self, link: &LinkView) -> Result<Vec<ImportOffer>, String> {
        let mut offers = Vec::new();
        for legacy in self.legacy_setlists(link)?.setlists {
            if link.catalog.setlists.iter().any(|s| s.id == legacy.id) {
                continue;
            }
            let resolved = resolve(&legacy, &link.catalog.project_songs);
            offers.push(ImportOffer {
                id: legacy.id,
                name: legacy.name,
                found: u32::try_from(resolved.setlist.entries.len()).unwrap_or(u32::MAX),
                missing: resolved.unresolved.into_iter().map(|i| i.name).collect(),
            });
        }
        Ok(offers)
    }

    /// v1's file for this project; none when v1 left no folder or no file for it.
    fn legacy_setlists(&self, link: &LinkView) -> Result<LegacyFile, String> {
        let Some(directory) = &self.legacy_directory else {
            return Ok(LegacyFile::default());
        };
        let file = directory.join(format!("{}.json", link.catalog.project_id));
        match LegacyFile::read(&file) {
            Err(ImportError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(LegacyFile::default())
            }
            other => other.map_err(|error| error.to_string()),
        }
    }
}
