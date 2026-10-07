use shared_kernel::{Bpm, Seconds};

use crate::Directive;

/// What the directives found in a song's window add up to. The first
/// `!length` and the first `!bpm` win (v1 behaviour); any `!1008` counts.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Directives {
    hard_stop: bool,
    length: Option<Seconds>,
    tempo: Option<Bpm>,
}

impl Directives {
    /// Folds directives in order of appearance.
    pub fn collect(found: impl IntoIterator<Item = Directive>) -> Self {
        let mut all = Self::default();
        for directive in found {
            match directive {
                Directive::HardStop => all.hard_stop = true,
                Directive::Length(seconds) => all.length = all.length.or(Some(seconds)),
                Directive::Tempo(bpm) => all.tempo = all.tempo.or(Some(bpm)),
            }
        }
        all
    }

    /// The song ends with a hard stop.
    pub fn hard_stop(&self) -> bool {
        self.hard_stop
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
