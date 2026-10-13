use std::sync::atomic::{AtomicUsize, Ordering};

/// How many clients are remembered; the app is one, a test or a second app a few more.
pub const EXIT_HANDLE_SLOTS: usize = 8;

/// The operating system handles of the connected clients, kept where a process exit hook can read
/// them without locks, threads or allocation. A slot holds the handle plus one, 0 means empty.
pub struct ExitHandles {
    slots: [AtomicUsize; EXIT_HANDLE_SLOTS],
}

impl ExitHandles {
    pub(super) const fn new() -> Self {
        Self {
            slots: [const { AtomicUsize::new(0) }; EXIT_HANDLE_SLOTS],
        }
    }

    pub(super) fn add(&self, handle: usize) {
        let wanted = handle.wrapping_add(1);
        for slot in &self.slots {
            if slot
                .compare_exchange(0, wanted, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return;
            }
        }
    }

    pub(super) fn remove(&self, handle: usize) {
        let wanted = handle.wrapping_add(1);
        for slot in &self.slots {
            let _ = slot.compare_exchange(wanted, 0, Ordering::SeqCst, Ordering::SeqCst);
        }
    }

    /// The handles of the clients connected now. Only atomics are touched, so it is safe to call
    /// while the process is exiting.
    #[must_use]
    pub fn snapshot(&self) -> [Option<usize>; EXIT_HANDLE_SLOTS] {
        let mut found = [None; EXIT_HANDLE_SLOTS];
        for (target, slot) in found.iter_mut().zip(&self.slots) {
            *target = slot.load(Ordering::SeqCst).checked_sub(1);
        }
        found
    }
}

/// The handles of every client of every server in this process.
pub static EXIT_HANDLES: ExitHandles = ExitHandles::new();
