//! Whether the performance is running, as far as the app can tell.

use protocol::{LinkView, Phase};

/// True while the show plays, counts in or hands over. Without live state (the link is down or
/// not yet connected) the app cannot know, so it says false.
#[must_use]
pub fn performance_runs(view: &LinkView) -> bool {
    view.live.as_ref().is_some_and(|live| {
        matches!(
            live.phase,
            Phase::Playing | Phase::CountingIn | Phase::HandingOver
        )
    })
}

#[cfg(test)]
mod tests {
    use protocol::{Live, Phase};

    use super::*;

    fn view(phase: Phase) -> LinkView {
        LinkView {
            live: Some(Live {
                phase,
                ..Live::default()
            }),
            ..LinkView::default()
        }
    }

    #[test]
    fn only_playing_counting_in_and_handing_over_count_as_running() {
        for phase in [Phase::Playing, Phase::CountingIn, Phase::HandingOver] {
            assert!(performance_runs(&view(phase)));
        }
        for phase in [
            Phase::Idle,
            Phase::Paused,
            Phase::HardStopped,
            Phase::Finished,
        ] {
            assert!(!performance_runs(&view(phase)));
        }
        assert!(!performance_runs(&LinkView::default()));
    }
}
