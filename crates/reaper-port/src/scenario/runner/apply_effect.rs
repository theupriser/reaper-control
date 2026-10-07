use performance::Effect;

use crate::ReaperPort;

/// Carries out one effect on REAPER (or the fake).
pub(super) fn apply_effect(reaper: &mut impl ReaperPort, effect: Effect) {
    match effect {
        Effect::Play => reaper.play(),
        Effect::Pause => reaper.pause(),
        Effect::SeekTo(position) => reaper.seek(position),
        Effect::SetCountIn(enabled) => reaper.set_count_in(enabled),
    }
}
