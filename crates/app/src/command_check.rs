//! The checks every command passes before it is queued.

use protocol::Command;

use crate::command_refusal::CommandRefusal;

/// Finds what is wrong with `command` without asking REAPER. Whether a song or setlist exists is
/// the extension's to judge; this only catches what can never be right.
pub fn check(command: &Command) -> Result<(), CommandRefusal> {
    match command {
        Command::Seek { position, .. } if !position.is_finite() || *position < 0.0 => {
            Err(CommandRefusal::InvalidPosition)
        }
        Command::SaveSetlist { id, .. } if id.trim().is_empty() => {
            Err(CommandRefusal::EmptySetlistId)
        }
        Command::SaveSetlist { name, .. } if name.trim().is_empty() => {
            Err(CommandRefusal::EmptySetlistName)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
