use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::CommandHandler;
use super::hub::Hub;

pub(super) struct Shared {
    pub(super) token: String,
    pub(super) handler: Box<dyn CommandHandler>,
    pub(super) extension_version: String,
    pub(super) hub: Mutex<Hub>,
    pub(super) stop: AtomicBool,
    pub(super) connections: AtomicUsize,
}

impl Shared {
    pub(super) fn hub(&self) -> MutexGuard<'_, Hub> {
        self.hub.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
