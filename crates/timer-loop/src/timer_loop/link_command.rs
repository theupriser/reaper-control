use performance::{Flag, Flags, Input};
use protocol::Command;
use shared_kernel::Seconds;

/// What the performance should do for a command from the link, or why it cannot yet.
pub(super) fn to_input(
    command: Command,
    song_start: Seconds,
    flags: Flags,
) -> Result<Input, &'static str> {
    match command {
        Command::Play => Ok(Input::Play),
        Command::Pause | Command::Stop => Ok(Input::Pause),
        Command::Next => Ok(Input::Next),
        Command::Previous => Ok(Input::Previous),
        Command::RestartSong => Ok(Input::RestartSong),
        Command::Seek {
            position,
            count_in: false,
        } => Seconds::new(song_start.get() + position)
            .map(|position| Input::Seek { position })
            .map_err(|_| "not a position"),
        Command::Seek { count_in: true, .. } => Err("count-in needs the tempo map"),
        Command::ToggleAutoResume => Ok(Input::SetFlag {
            flag: Flag::Autoplay,
            enabled: !flags.autoplay,
        }),
        Command::ToggleCountInOnMarker => Ok(Input::SetFlag {
            flag: Flag::CountIn,
            enabled: !flags.count_in,
        }),
        Command::ToggleRecordArm => Err("recording is not wired yet"),
        Command::SaveSetlist { .. } | Command::SetActiveSetlist { .. } => {
            Err("not a performance command")
        }
    }
}
