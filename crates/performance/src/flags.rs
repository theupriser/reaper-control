use crate::Flag;

/// The playback settings, all off by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags {
    /// Resume playing after Next, Previous and Restart.
    pub autoplay: bool,
    /// Count in when jumping to a cue.
    pub count_in: bool,
}

impl Flags {
    /// The current value of one flag.
    pub fn get(&self, flag: Flag) -> bool {
        match flag {
            Flag::Autoplay => self.autoplay,
            Flag::CountIn => self.count_in,
        }
    }

    /// Sets one flag; returns whether the value changed.
    pub fn set(&mut self, flag: Flag, enabled: bool) -> bool {
        let changed = self.get(flag) != enabled;
        match flag {
            Flag::Autoplay => self.autoplay = enabled,
            Flag::CountIn => self.count_in = enabled,
        }
        changed
    }
}
