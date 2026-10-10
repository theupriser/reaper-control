use shared_kernel::{Bpm, Seconds};

use crate::Directive;

/// What the directives found in a song's window add up to. The first
/// `!length` and the first `!bpm` win (v1 behaviour); any `!1008` counts.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Directives {
    hard_stop_at: Option<Seconds>,
    length: Option<Seconds>,
    tempo: Option<Bpm>,
}

impl Directives {
    /// Folds directives in order of appearance, each with the position of the cue it came from.
    pub fn collect(found: impl IntoIterator<Item = (Seconds, Directive)>) -> Self {
        let mut all = Self::default();
        for (position, directive) in found {
            match directive {
                Directive::HardStop => {
                    all.hard_stop_at = Some(all.hard_stop_at.map_or(position, |first| {
                        if position < first { position } else { first }
                    }));
                }
                Directive::Length(seconds) => all.length = all.length.or(Some(seconds)),
                Directive::Tempo(bpm) => all.tempo = all.tempo.or(Some(bpm)),
            }
        }
        all
    }

    /// The song ends with a hard stop.
    pub fn hard_stop(&self) -> bool {
        self.hard_stop_at.is_some()
    }

    /// Where the earliest hard stop marker lies: REAPER itself may stop there (SWS).
    pub fn hard_stop_at(&self) -> Option<Seconds> {
        self.hard_stop_at
    }

    /// Custom song length, when given.
    pub fn length(&self) -> Option<Seconds> {
        self.length
    }

    /// Declared tempo, when given.
    pub fn tempo(&self) -> Option<Bpm> {
        self.tempo
    }
}
