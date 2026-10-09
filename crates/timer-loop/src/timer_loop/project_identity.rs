use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use reaper_port::ReaperPort;

/// ExtState section and key the extension keeps the project id in.
pub(super) const SECTION: &str = "RC2";
pub(super) const KEY: &str = "project_id";
/// Where the project was saved when the id was written; a different path means the file was copied.
const PATH_KEY: &str = "project_path";
/// Where v1 kept its id; adopting it lets the app find the project's v1 setlist file. REAPER
/// writes section and key in capitals when it saves the project, so a saved project has both.
const LEGACY_SPELLINGS: [(&str, &str); 2] = [
    ("ReaperControl", "ProjectId"),
    ("REAPERCONTROL", "PROJECTID"),
];

/// Gives the project an id when it has none: the one v1 stored, otherwise a new random one. The
/// id is the key of the app's restore-only copy of the setlists (ADR-008, step 6). The saved path
/// is kept next to it: a file that was copied carries the original's id, so a different path gets
/// a new id (a moved or renamed file is not told apart from a copy and also gets one).
pub(super) fn ensure(port: &mut impl ReaperPort) {
    let path = port.project_path();
    let stored_path = port.ext_state(SECTION, PATH_KEY);
    let copied = matches!((&path, &stored_path), (Some(now), Some(then)) if now != then);
    if read(port).is_none() || copied {
        let id = if copied {
            generate()
        } else {
            LEGACY_SPELLINGS
                .iter()
                .filter_map(|(section, key)| port.ext_state(section, key))
                .find(|id| is_usable(id))
                .unwrap_or_else(generate)
        };
        port.set_ext_state(SECTION, KEY, &id);
    }
    if let Some(path) = path
        && stored_path.as_deref() != Some(path.as_str())
    {
        port.set_ext_state(SECTION, PATH_KEY, &path);
    }
}

/// The project's id, if it has one.
pub(super) fn read(port: &impl ReaperPort) -> Option<String> {
    port.ext_state(SECTION, KEY).filter(|id| is_usable(id))
}

fn is_usable(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn generate() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis());
    let random = RandomState::new().build_hasher().finish();
    format!("project-{millis}-{random:x}")
}
