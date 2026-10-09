mod count_in;
mod ext_state;
mod tempo;
mod timeline;

use std::time::Instant;

use performance::TempoMap;
use reaper_medium::{
    MainThreadScope, PositionInSeconds, ProjectContext, Reaper, SetEditCurPosOptions,
};
use reaper_port::{Marker, ReaperPort, Region, Transport};
use shared_kernel::Seconds;

/// The real `ReaperPort`: reads and drives the current project through `reaper-rs`. Main thread
/// only, because `Reaper<MainThreadScope>` cannot leave it.
pub(super) struct ReaperRsAdapter {
    reaper: Reaper<MainThreadScope>,
    started: Instant,
    /// The user's metronome settings while a count-in is armed.
    saved_metronome: Option<i32>,
}

impl ReaperRsAdapter {
    pub(super) fn new(reaper: Reaper<MainThreadScope>) -> Self {
        Self {
            reaper,
            started: Instant::now(),
            saved_metronome: None,
        }
    }

    /// The session's REAPER, for the probe's reference readings.
    #[cfg(feature = "probe")]
    pub(super) fn reaper(&self) -> &Reaper<MainThreadScope> {
        &self.reaper
    }

    fn project() -> ProjectContext {
        ProjectContext::CurrentProject
    }
}

impl std::fmt::Debug for ReaperRsAdapter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReaperRsAdapter")
            .finish_non_exhaustive()
    }
}

fn seconds(value: f64) -> Seconds {
    Seconds::new(value).unwrap_or(Seconds::ZERO)
}

impl ReaperPort for ReaperRsAdapter {
    fn now(&self) -> Seconds {
        seconds(self.started.elapsed().as_secs_f64())
    }

    fn position(&self) -> Seconds {
        let position = match self.transport() {
            Transport::Stopped => self
                .reaper
                .get_cursor_position_ex(Self::project())
                .map_or(0.0, PositionInSeconds::get),
            Transport::Playing | Transport::Paused => {
                self.reaper.get_play_position_2_ex(Self::project()).get()
            }
        };
        seconds(position)
    }

    fn transport(&self) -> Transport {
        let state = self.reaper.get_play_state_ex(Self::project());
        if state.is_paused {
            Transport::Paused
        } else if state.is_playing {
            Transport::Playing
        } else {
            Transport::Stopped
        }
    }

    fn change_count(&self) -> u64 {
        u64::from(self.reaper.get_project_state_change_count(Self::project()))
    }

    fn project_token(&self) -> u64 {
        // SAFETY: a negative index asks for the current project; no name buffer is wanted.
        let project = unsafe { self.reaper.low().EnumProjects(-1, std::ptr::null_mut(), 0) };
        project as usize as u64
    }

    fn count_in(&self) -> bool {
        count_in::read(&self.reaper, self.saved_metronome)
    }

    fn regions(&self) -> Vec<Region> {
        timeline::read(&self.reaper).0
    }

    fn markers(&self) -> Vec<Marker> {
        timeline::read(&self.reaper).1
    }

    fn tempo_map(&self) -> TempoMap {
        tempo::read(&self.reaper)
    }

    fn ext_state(&self, section: &str, key: &str) -> Option<String> {
        ext_state::read(&self.reaper, section, key)
    }

    fn play(&mut self) {
        if self.transport() != Transport::Playing {
            self.reaper.on_play_button_ex(Self::project());
        }
    }

    fn pause(&mut self) {
        if self.transport() == Transport::Playing {
            self.reaper.on_pause_button_ex(Self::project());
        }
    }

    fn seek(&mut self, position: Seconds) {
        if let Ok(position) = PositionInSeconds::new(position.get()) {
            self.reaper.set_edit_curs_pos_2(
                Self::project(),
                position,
                SetEditCurPosOptions {
                    move_view: false,
                    seek_play: true,
                },
            );
        }
    }

    fn set_count_in(&mut self, enabled: bool) {
        count_in::write(&self.reaper, &mut self.saved_metronome, enabled);
    }

    fn set_ext_state(&mut self, section: &str, key: &str, value: &str) {
        ext_state::write(&self.reaper, section, key, value);
    }
}
