//! Stand-in for the performance core until the extension link exists (Phase 2).
//! One pure reducer: every UI intent becomes a [`Command`] and passes through [`dispatch`].

pub use protocol::{AppState, Command, Phase};

/// The single entry point for every mutation.
#[must_use]
pub fn dispatch(state: AppState, command: Command) -> AppState {
    match command {
        Command::Play => AppState {
            phase: Phase::Playing,
            ..state
        },
        Command::Pause if state.phase == Phase::Playing => AppState {
            phase: Phase::Paused,
            ..state
        },
        Command::Pause => state,
        Command::Next | Command::Previous | Command::RestartSong => state,
        Command::Stop => AppState {
            phase: Phase::Idle,
            position: 0.0,
            ..state
        },
        Command::Seek { position, .. } => AppState {
            position: if position.is_finite() {
                position.max(0.0)
            } else {
                state.position
            },
            ..state
        },
        Command::ToggleAutoResume => AppState {
            auto_resume: !state.auto_resume,
            ..state
        },
        Command::ToggleCountInOnMarker => AppState {
            count_in_on_marker: !state.count_in_on_marker,
            ..state
        },
        Command::ToggleRecordArm => AppState {
            record_armed: !state.record_armed,
            ..state
        },
    }
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
            ..AppState::default()
        };
        assert_eq!(dispatch(paused, Command::Play).phase, Phase::Playing);
    }

    #[test]
    fn seek_moves_the_position_and_keeps_the_phase() {
        let playing = dispatch(AppState::default(), Command::Play);
        let seeked = dispatch(
            playing,
            Command::Seek {
                position: 64.0,
                count_in: false,
            },
        );
        assert_eq!(seeked.phase, Phase::Playing);
        assert!((seeked.position - 64.0).abs() < f64::EPSILON);
    }

    #[test]
    fn seek_refuses_negative_and_non_finite_positions() {
        let at = |position| {
            dispatch(
                dispatch(
                    AppState::default(),
                    Command::Seek {
                        position: 10.0,
                        count_in: false,
                    },
                ),
                Command::Seek {
                    position,
                    count_in: false,
                },
            )
            .position
        };
        assert!(at(-5.0).abs() < f64::EPSILON);
        assert!((at(f64::NAN) - 10.0).abs() < f64::EPSILON);
        assert!((at(f64::INFINITY) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stop_rewinds_to_the_start() {
        let seeked = dispatch(
            AppState::default(),
            Command::Seek {
                position: 30.0,
                count_in: false,
            },
        );
        assert!(dispatch(seeked, Command::Stop).position.abs() < f64::EPSILON);
    }

    #[test]
    fn toggles_flip_only_their_own_flag() {
        let start = AppState::default();
        let a = dispatch(start, Command::ToggleAutoResume);
        assert_eq!(a.auto_resume, !start.auto_resume);
        assert_eq!(a.count_in_on_marker, start.count_in_on_marker);
        let c = dispatch(a, Command::ToggleCountInOnMarker);
        assert_eq!(c.count_in_on_marker, !start.count_in_on_marker);
        let r = dispatch(c, Command::ToggleRecordArm);
        assert!(r.record_armed);
        assert!(!dispatch(r, Command::ToggleRecordArm).record_armed);
    }
}
