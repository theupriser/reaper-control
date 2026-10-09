/// Why a command was not carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// The setlist has no songs.
    NothingToPlay,
    /// The current song is the last one.
    NoNextSong,
    /// The current song is the first one.
    NoPreviousSong,
    /// The setlist has no song at that position.
    NoSuchSong,
    /// The target position is outside the current song.
    OutsideSong,
    /// The command makes no sense in the current phase.
    NotNow,
}
