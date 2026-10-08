//! The restore-only copy of the project's setlists (D2).

use std::path::PathBuf;

use protocol::SetlistInfo;

use crate::mirror_error::MirrorError;
use crate::mirror_file::MirrorFile;

const SCHEMA_VERSION: u32 = 1;

/// One file per project in a folder. It is only read to offer a restore, never to overwrite the project.
#[derive(Debug, Clone)]
pub struct SetlistMirror {
    directory: PathBuf,
}

impl SetlistMirror {
    /// A mirror in this folder; nothing is touched yet.
    #[must_use]
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    fn file(&self, project_id: &str) -> Result<PathBuf, MirrorError> {
        let safe = !project_id.is_empty()
            && project_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !safe {
            return Err(MirrorError::BadProjectId(project_id.into()));
        }
        Ok(self.directory.join(format!("{project_id}.json")))
    }

    /// Replaces the copy of this project's setlists, through a temporary file and a rename.
    ///
    /// # Errors
    /// [`MirrorError`] for an unusable project id or a failed write.
    pub fn save(&self, project_id: &str, setlists: &[SetlistInfo]) -> Result<(), MirrorError> {
        let file = self.file(project_id)?;
        std::fs::create_dir_all(&self.directory)?;
        let content = MirrorFile {
            schema_version: SCHEMA_VERSION,
            setlists: setlists.to_vec(),
        };
        let temporary = file.with_extension("json.tmp");
        std::fs::write(&temporary, serde_json::to_vec_pretty(&content)?)?;
        std::fs::rename(&temporary, &file)?;
        Ok(())
    }

    /// The copy of this project's setlists; empty when there is none.
    ///
    /// # Errors
    /// [`MirrorError`] for an unusable project id or a damaged file.
    pub fn restore(&self, project_id: &str) -> Result<Vec<SetlistInfo>, MirrorError> {
        let text = match std::fs::read_to_string(self.file(project_id)?) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        Ok(serde_json::from_str::<MirrorFile>(&text)?.setlists)
    }
}
