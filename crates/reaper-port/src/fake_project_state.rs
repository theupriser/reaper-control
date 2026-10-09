use std::collections::BTreeMap;

/// What `FakeReaper` remembers about the project besides its songs: the ExtState, the change
/// counter, the project token and the file path. Kept behind a `Box` so `FakeReaper` stays small.
#[derive(Debug, Clone)]
pub(crate) struct FakeProjectState {
    pub(crate) ext_state: BTreeMap<(String, String), String>,
    pub(crate) change_count: u64,
    pub(crate) project_token: u64,
    pub(crate) project_path: Option<String>,
}

impl Default for FakeProjectState {
    fn default() -> Self {
        Self {
            ext_state: BTreeMap::new(),
            change_count: 0,
            project_token: 1,
            project_path: None,
        }
    }
}
