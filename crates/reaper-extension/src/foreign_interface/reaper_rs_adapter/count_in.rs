use reaper_medium::{MainThreadScope, Reaper};

/// The project's metronome preferences as bits (the dialog "Metronome and pre-roll settings").
/// REAPER has no action that toggles the count-in (action 40363 only opens the dialog). Measured
/// in REAPER 7.82: "Count-in before playback" does nothing unless "Enable metronome" is also on,
/// and the click then plays during the count-in only if "Run metronome during playback" is off.
const METRONOME_SETTINGS: &str = "projmetroen";
const ENABLED: i32 = 1;
const DURING_PLAYBACK: i32 = 2;
const COUNT_IN_BEFORE_PLAYBACK: i32 = 8;
const SETTING_SIZE: u32 = 4;

/// Whether the user has "Count-in before playback" checked. While the extension has armed a
/// count-in this reads what the user had.
pub(super) fn read(reaper: &Reaper<MainThreadScope>, saved: Option<i32>) -> bool {
    saved
        .or_else(|| current(reaper))
        .is_some_and(|settings| settings & COUNT_IN_BEFORE_PLAYBACK != 0)
}

/// Arms a count-in for the next start of playback (the metronome on, the count-in on, no click
/// during playback), keeping the user's settings in `saved`; or puts the user's settings back.
pub(super) fn write(reaper: &Reaper<MainThreadScope>, saved: &mut Option<i32>, enabled: bool) {
    let Some(pointer) = pointer(reaper) else {
        return;
    };
    // SAFETY: REAPER owns the setting for the whole session and it is 4 bytes, an `i32`; this is
    // the main thread, the only one REAPER changes it from.
    unsafe {
        if enabled {
            let settings = *saved.get_or_insert(*pointer);
            *pointer = (settings | ENABLED | COUNT_IN_BEFORE_PLAYBACK) & !DURING_PLAYBACK;
        } else if let Some(settings) = saved.take() {
            *pointer = settings;
        }
    }
}

fn current(reaper: &Reaper<MainThreadScope>) -> Option<i32> {
    // SAFETY: as in `write`.
    pointer(reaper).map(|pointer| unsafe { *pointer })
}

fn pointer(reaper: &Reaper<MainThreadScope>) -> Option<*mut i32> {
    reaper
        .get_config_var(METRONOME_SETTINGS)
        .filter(|found| found.size == SETTING_SIZE)
        .map(|found| found.value.as_ptr().cast::<i32>())
}
