//! Keeps the restore-only copy of the project's setlists up to date.

use std::sync::{Arc, Mutex};

use protocol::{LinkView, SetlistInfo};

use crate::mirror_repository::MirrorRepository;

/// Writes the project's setlists to the mirror whenever they change. An empty list is never
/// written: a project that lost its setlists is exactly when the copy is needed.
pub struct MirrorKeeper {
    mirror: Arc<dyn MirrorRepository>,
    written: Mutex<Option<(String, Vec<SetlistInfo>)>>,
}

impl MirrorKeeper {
    /// A keeper that writes to `mirror`.
    #[must_use]
    pub fn new(mirror: Arc<dyn MirrorRepository>) -> Self {
        Self {
            mirror,
            written: Mutex::new(None),
        }
    }

    /// Looks at a new view and writes the setlists when they differ from the last write.
    pub fn observe(&self, view: &LinkView) {
        let catalog = &view.catalog;
        if catalog.project_id.is_empty() || catalog.setlists.is_empty() {
            return;
        }
        let Ok(mut written) = self.written.lock() else {
            return;
        };
        let current = (catalog.project_id.clone(), catalog.setlists.clone());
        if written.as_ref() == Some(&current) {
            return;
        }
        match self.mirror.save(&current.0, &current.1) {
            Ok(()) => *written = Some(current),
            Err(error) => tracing::warn!(%error, "setlist mirror not written"),
        }
    }
}

#[cfg(test)]
mod tests;
