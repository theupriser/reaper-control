use catalogue::{Directive, MarkerName};
use protocol::MarkerReading;

/// Reads a marker or region name with the same grammar the extension uses.
pub fn read_marker_name(name: &str) -> MarkerReading {
    let parsed = MarkerName::parse(name);
    let mut reading = MarkerReading {
        hidden: parsed.is_hidden(),
        ..MarkerReading::default()
    };
    for directive in parsed.directives() {
        match directive {
            Directive::HardStop => reading.hard_stop = true,
            Directive::Length(seconds) => reading.length_seconds = Some(seconds.get()),
            Directive::Tempo(tempo) => reading.tempo_bpm = Some(tempo.get()),
        }
    }
    reading
}
