//! The MIDI notes become bindings that a press is looked up in.

use app_core::{ControlBinding, ControlSource, Intent, MidiConfig};

#[test]
fn every_note_becomes_a_binding_to_its_intent() {
    let config = MidiConfig::default();
    let bindings = config.bindings();
    assert_eq!(bindings.len(), config.notes.len());
    assert_eq!(
        ControlBinding::intent_for(&bindings, ControlSource::MidiNote { note: 51 }),
        Some(Intent::Next)
    );
}

#[test]
fn an_unbound_note_has_no_intent() {
    let bindings = MidiConfig::default().bindings();
    assert_eq!(
        ControlBinding::intent_for(&bindings, ControlSource::MidiNote { note: 0 }),
        None
    );
}

#[test]
fn a_changed_mapping_changes_the_bindings() {
    let mut config = MidiConfig::default();
    config.notes.clear();
    config.notes.insert(60, Intent::Pause);
    assert_eq!(
        config.bindings(),
        vec![ControlBinding {
            source: ControlSource::MidiNote { note: 60 },
            intent: Intent::Pause,
        }]
    );
}
