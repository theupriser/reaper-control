use shared_kernel::{Seconds, SongId};

use crate::{Cue, Directives, InvalidSong};

/// A region of the project that can be played as a song.
#[derive(Debug, Clone, PartialEq)]
pub struct Song {
    id: SongId,
    name: String,
    start: Seconds,
    end: Seconds,
    window_length: Seconds,
}

impl Song {
    /// A song needs a window with a positive length.
    pub fn new(
        id: SongId,
        name: impl Into<String>,
        start: Seconds,
        end: Seconds,
    ) -> Result<Self, InvalidSong> {
        if end <= start {
            return Err(InvalidSong::EmptyWindow);
        }
        let window_length = Seconds::new(end.get() - start.get())?;
        Ok(Self {
            id,
            name: name.into(),
            start,
            end,
            window_length,
        })
    }

    /// Identity.
    pub fn id(&self) -> &SongId {
        &self.id
    }

    /// Region name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Window start.
    pub fn start(&self) -> Seconds {
        self.start
    }

    /// Window end.
    pub fn end(&self) -> Seconds {
        self.end
    }

    /// Whether a position lies in the window, edges included (v1).
    pub fn contains(&self, position: Seconds) -> bool {
        position >= self.start && position <= self.end
    }

    /// Directives of the cues inside this song's window.
    pub fn directives(&self, cues: &[Cue]) -> Directives {
        Directives::collect(
            cues.iter()
                .filter(|cue| self.contains(cue.position()))
                .flat_map(|cue| cue.parsed().directives().iter().copied()),
        )
    }

    /// Effective length: `!length` when given, else the window.
    pub fn length(&self, directives: &Directives) -> Seconds {
        directives.length().unwrap_or(self.window_length)
    }
}
