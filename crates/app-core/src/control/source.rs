/// Where a press comes from. Keyboard keys and remote clients join this list when they
/// are bound from the app's side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSource {
    /// A MIDI note, 0 to 127, on any channel the settings allow.
    MidiNote {
        /// The note number.
        note: u8,
    },
}
