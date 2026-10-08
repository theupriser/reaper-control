//! Turns every panic in the app into a log line and a call to `on_panic`, then lets the thread end as usual.

use std::panic::PanicHookInfo;

/// Describes a panic as `message at file:line on thread`.
#[must_use]
pub fn describe(info: &PanicHookInfo<'_>) -> String {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_owned());
    let place = info.location().map_or_else(String::new, |location| {
        format!(" at {}:{}", location.file(), location.line())
    });
    let thread = std::thread::current();
    format!(
        "{message}{place} on {}",
        thread.name().unwrap_or("unnamed thread")
    )
}

/// Installs the hook for the whole process; the last call wins.
pub fn install_panic_hook(on_panic: impl Fn(&str) + Send + Sync + 'static) {
    std::panic::set_hook(Box::new(move |info| {
        let text = describe(info);
        tracing::error!(panic = %text, "the app panicked");
        on_panic(&text);
    }));
}
