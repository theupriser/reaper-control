use performance::PlannedSong;
use protocol::{EntryInfo, SetlistInfo};
use reaper_port::ReaperPort;
use setlists::{Entry, EntryId, Revision, Setlist, SetlistId};
use shared_kernel::SongId;

/// ExtState section the extension keeps its project data in.
const SECTION: &str = "RC2";
/// The setlists of the project, a JSON list of `SetlistInfo`.
const SETLISTS_KEY: &str = "setlists";
/// The id of the setlist that is played; without one the songs play in timeline order.
const ACTIVE_KEY: &str = "active_setlist";

/// The setlists stored in the project's ExtState and which one is played. A setlist that does not
/// hold together (empty name, repeated entry id) is left out, as is stored text that is not JSON.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct ProjectSetlists {
    setlists: Vec<SetlistInfo>,
    active: Option<String>,
    stored_text: Option<String>,
}

impl ProjectSetlists {
    pub(super) fn read(port: &impl ReaperPort) -> Self {
        let stored_text = port.ext_state(SECTION, SETLISTS_KEY);
        let stored: Vec<SetlistInfo> = stored_text
            .as_deref()
            .and_then(|text| serde_json::from_str(text).ok())
            .unwrap_or_default();
        Self {
            setlists: stored.into_iter().filter(Self::holds_together).collect(),
            active: Self::read_active(port),
            stored_text,
        }
    }

    /// Whether the project still holds what this was read from. REAPER's project change count does
    /// not move for ExtState, so the stored text itself is compared (two short reads).
    pub(super) fn is_current(&self, port: &impl ReaperPort) -> bool {
        port.ext_state(SECTION, SETLISTS_KEY) == self.stored_text
            && Self::read_active(port) == self.active
    }

    fn read_active(port: &impl ReaperPort) -> Option<String> {
        port.ext_state(SECTION, ACTIVE_KEY)
            .filter(|id| !id.is_empty())
    }

    pub(super) fn setlists(&self) -> &[SetlistInfo] {
        &self.setlists
    }

    /// The id of the played setlist, if it exists.
    pub(super) fn active(&self) -> Option<&str> {
        self.active_setlist().map(|setlist| setlist.id.as_str())
    }

    /// The planned songs in the order of the played setlist; without one, as they are. Entries
    /// whose song the project no longer has are skipped, a song may appear more than once.
    pub(super) fn arrange(&self, timeline: Vec<PlannedSong>) -> Vec<PlannedSong> {
        let Some(setlist) = self.active_setlist() else {
            return timeline;
        };
        setlist
            .entries
            .iter()
            .filter_map(|entry| timeline.iter().find(|song| song.song_id == entry.song_id))
            .cloned()
            .collect()
    }

    fn active_setlist(&self) -> Option<&SetlistInfo> {
        let active = self.active.as_deref()?;
        self.setlists.iter().find(|setlist| setlist.id == active)
    }

    fn holds_together(info: &SetlistInfo) -> bool {
        let entries = info
            .entries
            .iter()
            .map(|EntryInfo { id, song_id }| Entry::new(EntryId::new(*id), SongId::new(song_id)))
            .collect();
        Setlist::restore(
            SetlistId::new(&info.id),
            &info.name,
            entries,
            Revision::new(info.revision),
        )
        .is_ok()
    }
}
