//! Stand-in for the performance core until the extension link exists (Phase 2).
//! One pure reducer: every UI intent becomes a [`Command`] and passes through [`dispatch`].

use serde::{Deserialize, Serialize};

/// Phases the stub knows about; a subset of SPEC §14.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Phase {
    /// Nothing is playing.
    Idle,
    /// Playing a song.
    Playing,
    /// Playback is paused.
    Paused,
}

/// Everything the UI can ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Command {
    /// Start or resume playback.
    Play,
    /// Pause playback.
    Pause,
    /// Stop playback.
    Stop,
}

/// What the UI renders. The UI never infers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AppState {
    /// Current phase.
    pub phase: Phase,
}

impl Default for AppState {
    fn default() -> Self {
        Self { phase: Phase::Idle }
    }
}

/// The single entry point for every mutation.
#[must_use]
pub fn dispatch(state: AppState, command: Command) -> AppState {
    let phase = match (state.phase, command) {
        (_, Command::Play) => Phase::Playing,
        (Phase::Playing, Command::Pause) => Phase::Paused,
        (phase, Command::Pause) => phase,
        (_, Command::Stop) => Phase::Idle,
    };
    AppState { phase }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_then_pause_then_stop() {
        let playing = dispatch(AppState::default(), Command::Play);
        assert_eq!(playing.phase, Phase::Playing);
        let paused = dispatch(playing, Command::Pause);
        assert_eq!(paused.phase, Phase::Paused);
        assert_eq!(dispatch(paused, Command::Stop).phase, Phase::Idle);
    }

    #[test]
    fn pause_while_idle_stays_idle() {
        assert_eq!(
            dispatch(AppState::default(), Command::Pause).phase,
            Phase::Idle
        );
    }

    #[test]
    fn play_resumes_from_pause() {
        let paused = AppState {
            phase: Phase::Paused,
        };
        assert_eq!(dispatch(paused, Command::Play).phase, Phase::Playing);
    }
}
