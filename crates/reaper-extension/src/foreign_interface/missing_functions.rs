use std::ffi::CString;

use reaper_low::PluginContext;
use thiserror::Error;

use super::required_functions::REQUIRED_FUNCTIONS;

/// The running REAPER lacks functions the extension needs, so it is too old (ADR-009).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("this REAPER lacks functions the extension needs: {}", names.join(", "))]
pub(super) struct MissingFunctions {
    names: Vec<&'static str>,
}

impl MissingFunctions {
    /// Asks REAPER for every required function.
    pub(super) fn check(context: PluginContext) -> Result<(), Self> {
        Self::among(REQUIRED_FUNCTIONS, |name| {
            CString::new(name)
                // SAFETY: `GetFunc` only looks the name up and returns null when it is unknown.
                .is_ok_and(|name| !unsafe { context.GetFunc(name.as_ptr()) }.is_null())
        })
    }

    /// The names for which `is_present` says no; an error when there is any.
    fn among(names: &[&'static str], is_present: impl Fn(&str) -> bool) -> Result<(), Self> {
        let missing: Vec<_> = names
            .iter()
            .copied()
            .filter(|name| !is_present(name))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(Self { names: missing })
        }
    }
}

#[cfg(test)]
mod tests;
