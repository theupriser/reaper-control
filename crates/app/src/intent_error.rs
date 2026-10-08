//! Why an intent did not go out.

use crate::dispatch_error::DispatchError;
use crate::intent_refusal::IntentRefusal;

/// Why `IntentDispatcher::dispatch` did not send anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IntentError {
    /// The state says the intent is pointless.
    #[error(transparent)]
    Refused(#[from] IntentRefusal),
    /// The bus did not send the command.
    #[error(transparent)]
    Dispatch(#[from] DispatchError),
}
