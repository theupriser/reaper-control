use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use reaper_port::ReaperPort;

/// ExtState section and key the extension keeps the project id in.
pub(super) const SECTION: &str = "RC2";
pub(super) const KEY: &str = "project_id";
/// Where v1 kept its id; adopting it lets the app find the project's v1 setlist file. REAPER
/// writes section and key in capitals when it saves the project, so a saved project has both.
const LEGACY_SPELLINGS: [(&str, &str); 2] = [
    ("ReaperControl", "ProjectId"),
    ("REAPERCONTROL", "PROJECTID"),
];

/// Gives the project an id when it has none: the one v1 stored, otherwise a new random one. The
/// id is the key of the app's restore-only copy of the setlists (ADR-008, step 6).
pub(super) fn ensure(port: &mut impl ReaperPort) {
    if read(port).is_some() {
        return;
    }
    let id = LEGACY_SPELLINGS
        .iter()
        .filter_map(|(section, key)| port.ext_state(section, key))
        .find(|id| is_usable(id))
        .unwrap_or_else(generate);
    port.set_ext_state(SECTION, KEY, &id);
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
