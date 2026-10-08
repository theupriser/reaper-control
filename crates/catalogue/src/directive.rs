use shared_kernel::{Bpm, Seconds};

/// One special token in a marker name (SPEC §4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Directive {
    /// `!1008` or `!hardstop`: pause (hard stop) when playback reaches the song's end.
    HardStop,
    /// `!length:N`: the song lasts N seconds instead of its region length.
    Length(Seconds),
    /// `!bpm:N`: the song's tempo, used for the count-in.
    Tempo(Bpm),
}

impl Directive {
    /// Classifies one whitespace-free token. `None` when it is not one of ours
    /// (also for a malformed number, so the token stays plain text).
    pub fn parse(token: &str) -> Option<Self> {
        if token == "!1008" || token == "!hardstop" {
            return Some(Self::HardStop);
        }
        if let Some(number) = token.strip_prefix("!length:") {
            let seconds = Seconds::new(plain_number(number)?).ok()?;
            return (seconds.get() > 0.0).then_some(Self::Length(seconds));
        }
        let number = token.strip_prefix("!bpm:")?;
        Bpm::new(plain_number(number)?).ok().map(Self::Tempo)
    }
}

/// `digits` or `digits.digits`, nothing else (v1: `\d+(\.\d+)?`).
fn plain_number(text: &str) -> Option<f64> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, "0"));
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    if digits(whole) && digits(fraction) {
        text.parse().ok()
    } else {
        None
    }
}
