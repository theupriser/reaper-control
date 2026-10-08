use reaper_medium::{MainThreadScope, Reaper};

/// The project's metronome preferences as bits; REAPER has no action that toggles the count-in
/// (action 40363 only opens the settings dialog, checked in REAPER 7.82).
const METRONOME_SETTINGS: &str = "projmetroen";
const COUNT_IN_BEFORE_PLAYBACK: i32 = 8;
const SETTING_SIZE: u32 = 4;

/// Whether "Count-in before playback" is checked.
pub(super) fn read(reaper: &Reaper<MainThreadScope>) -> bool {
    reaper
        .get_config_var(METRONOME_SETTINGS)
        .filter(|found| found.size == SETTING_SIZE)
        // SAFETY: REAPER owns the setting for the whole session and it is 4 bytes, an `i32`.
        .is_some_and(
            |found| unsafe { *found.value.as_ptr().cast::<i32>() } & COUNT_IN_BEFORE_PLAYBACK != 0,
        )
}

/// Checks or unchecks "Count-in before playback", leaving every other setting as it is.
pub(super) fn write(reaper: &Reaper<MainThreadScope>, enabled: bool) {
    let Some(found) = reaper
        .get_config_var(METRONOME_SETTINGS)
        .filter(|found| found.size == SETTING_SIZE)
    else {
        return;
    };
    let pointer = found.value.as_ptr().cast::<i32>();
    // SAFETY: as in `read`; this is the main thread, the only one REAPER changes it from.
    unsafe {
        let settings = *pointer;
        *pointer = if enabled {
            settings | COUNT_IN_BEFORE_PLAYBACK
        } else {
            settings & !COUNT_IN_BEFORE_PLAYBACK
        };
    }
}
