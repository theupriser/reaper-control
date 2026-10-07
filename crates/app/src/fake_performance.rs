//! Stand-in for the performance core until the extension link exists (Phase 2).
//! One pure reducer: every UI intent becomes a [`Command`] and passes through [`dispatch`].

pub use protocol::{AppState, Command, Phase};

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
