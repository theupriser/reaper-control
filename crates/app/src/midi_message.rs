//! A raw MIDI message, as far as the app cares.

/// What the app understands of the bytes a device sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiMessage {
    /// A key was pressed (velocity 0 is how some devices say it was released).
    NoteOn {
        /// Channel 0 to 15.
        channel: u8,
        /// Note 0 to 127.
        note: u8,
        /// Velocity 0 to 127.
        velocity: u8,
    },
}

impl MidiMessage {
    /// The message in these bytes; `None` for anything but a note-on.
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [status, note, velocity] if status & 0xF0 == 0x90 => Some(Self::NoteOn {
                channel: status & 0x0F,
                note: *note,
                velocity: *velocity,
            }),
            _ => None,
        }
    }
}
