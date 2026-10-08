use protocol::{Live, Phase};

use super::*;

fn view(phase: Phase, current: Option<u32>, next: Option<u32>, setlist: bool) -> LinkView {
    LinkView {
        live: Some(Live {
            phase,
            current_song: current,
            next_song: next,
            setlist_id: setlist.then(|| "set".to_string()),
            ..Live::default()
        }),
        ..LinkView::default()
    }
}

#[test]
fn every_intent_becomes_its_command() {
    let ready = view(Phase::Idle, Some(0), Some(1), true);
    let pairs = [
        (Intent::RestartSong, Command::RestartSong),
        (Intent::ToggleAutoResume, Command::ToggleAutoResume),
        (
            Intent::ToggleCountInOnMarker,
            Command::ToggleCountInOnMarker,
        ),
        (Intent::ToggleRecordArm, Command::ToggleRecordArm),
        (Intent::Previous, Command::Previous),
        (Intent::Pause, Command::Pause),
        (Intent::TogglePlay, Command::Play),
        (Intent::Next, Command::Next),
    ];
    for (intent, command) in pairs {
        assert_eq!(IntentTranslator::translate(intent, &ready), Ok(command));
    }
}

#[test]
fn the_play_toggle_pauses_only_while_the_performance_runs() {
    let running = view(Phase::Playing, Some(0), Some(1), true);
    assert_eq!(
        IntentTranslator::translate(Intent::TogglePlay, &running),
        Ok(Command::Pause)
    );
}

#[test]
fn moving_without_a_current_song_or_past_the_last_song_is_refused() {
    let nothing = view(Phase::Idle, None, None, true);
    for intent in [Intent::Previous, Intent::Next, Intent::RestartSong] {
        assert_eq!(
            IntentTranslator::translate(intent, &nothing),
            Err(IntentRefusal::NothingToPlay)
        );
    }
    let last = view(Phase::Playing, Some(2), None, true);
    assert_eq!(
        IntentTranslator::translate(Intent::Next, &last),
        Err(IntentRefusal::NoNextSong)
    );
    assert_eq!(
        IntentTranslator::translate(Intent::Previous, &last),
        Ok(Command::Previous)
    );
}

#[test]
fn without_a_setlist_or_without_live_state_the_extension_decides() {
    let free = view(Phase::Idle, Some(0), None, false);
    assert_eq!(
        IntentTranslator::translate(Intent::Next, &free),
        Ok(Command::Next)
    );
    assert_eq!(
        IntentTranslator::translate(Intent::Next, &LinkView::default()),
        Ok(Command::Next)
    );
}
