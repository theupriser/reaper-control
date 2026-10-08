//! From an intent to the command that carries it out, judged against what the app last saw.

use protocol::{Command, LinkView};

use crate::intent::Intent;
use crate::intent_refusal::IntentRefusal;
use crate::performance_runs::performance_runs;

/// The one place an intent is checked against the current state and becomes a command. Without
/// live state the app cannot judge, so it lets the command through for the extension to decide.
#[derive(Debug, Clone, Copy, Default)]
pub struct IntentTranslator;

impl IntentTranslator {
    /// The command for `intent`, or why the state makes it pointless.
    pub fn translate(intent: Intent, view: &LinkView) -> Result<Command, IntentRefusal> {
        if let Some(live) = &view.live {
            let moves = matches!(
                intent,
                Intent::Previous | Intent::Next | Intent::RestartSong
            );
            if moves && live.current_song.is_none() {
                return Err(IntentRefusal::NothingToPlay);
            }
            if intent == Intent::Next && live.setlist_id.is_some() && live.next_song.is_none() {
                return Err(IntentRefusal::NoNextSong);
            }
        }
        Ok(match intent {
            Intent::RestartSong => Command::RestartSong,
            Intent::ToggleAutoResume => Command::ToggleAutoResume,
            Intent::ToggleCountInOnMarker => Command::ToggleCountInOnMarker,
            Intent::ToggleRecordArm => Command::ToggleRecordArm,
            Intent::Previous => Command::Previous,
            Intent::Pause => Command::Pause,
            Intent::TogglePlay if performance_runs(view) => Command::Pause,
            Intent::TogglePlay => Command::Play,
            Intent::Next => Command::Next,
        })
    }
}

#[cfg(test)]
mod tests;
