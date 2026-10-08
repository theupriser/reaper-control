use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use reaper_medium::{MainThreadScope, ProjectContext, Reaper};
use reaper_port::{Marker, Region};
use shared_kernel::{Seconds, SongId};

/// What the project holds at one index of REAPER's list of markers and regions.
enum Entry {
    Region(Region),
    Marker(Marker),
}

/// Every region and every marker of the current project, each list in timeline order.
pub(super) fn read(reaper: &Reaper<MainThreadScope>) -> (Vec<Region>, Vec<Marker>) {
    let mut regions = Vec::new();
    let mut markers = Vec::new();
    for index in 0.. {
        let Some(entry) = entry_at(reaper, index) else {
            break;
        };
        match entry {
            Some(Entry::Region(region)) => regions.push(region),
            Some(Entry::Marker(marker)) => markers.push(marker),
            None => {}
        }
    }
    regions.sort_by(|a, b| {
        a.start
            .partial_cmp(&b.start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    markers.sort_by(|a, b| {
        a.position
            .partial_cmp(&b.position)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    (regions, markers)
}

/// `None` past the last index; `Some(None)` for an entry with a position REAPER cannot report.
fn entry_at(reaper: &Reaper<MainThreadScope>, index: u32) -> Option<Option<Entry>> {
    reaper.enum_project_markers_3(ProjectContext::CurrentProject, index, |found| {
        let found = found?;
        let name = found.name.to_str().to_string();
        let start = Seconds::new(found.position.get()).ok();
        Some(match found.region_end_position {
            Some(end) => start.zip(Seconds::new(end.get()).ok()).map(|(start, end)| {
                Entry::Region(Region {
                    id: SongId::new(guid(reaper, index)),
                    name,
                    start,
                    end,
                })
            }),
            None => start.map(|position| Entry::Marker(Marker { name, position })),
        })
    })
}

/// The GUID of the marker or region at `index` (ADR-008); empty when REAPER has none.
fn guid(reaper: &Reaper<MainThreadScope>, index: u32) -> String {
    let Ok(description) = CString::new(format!("MARKER_GUID:{index}")) else {
        return String::new();
    };
    let mut buffer: Vec<c_char> = vec![0; 64];
    // SAFETY: the description and the buffer outlive the call; REAPER writes a NUL-terminated
    // string of at most 64 bytes into the buffer.
    let found = unsafe {
        reaper.low().GetSetProjectInfo_String(
            ProjectContext::CurrentProject.to_raw(),
            description.as_ptr(),
            buffer.as_mut_ptr(),
            false,
        )
    };
    if !found {
        return String::new();
    }
    // SAFETY: see above.
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}
