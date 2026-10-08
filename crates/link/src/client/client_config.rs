use std::time::Duration;

use crate::Endpoint;

/// How the client finds and watches the extension.
pub struct ClientConfig {
    /// Called before every connection attempt; returns the current endpoint.
    pub endpoint: Box<dyn Fn() -> Option<Endpoint> + Send>,
    /// First wait after a failure; doubles up to `max_backoff`.
    pub min_backoff: Duration,
    /// Longest wait between attempts.
    pub max_backoff: Duration,
    /// Send a Ping after this much silence.
    pub ping_after: Duration,
    /// Report the link as quiet after this much silence (the ping has had time to be answered).
    pub quiet_after: Duration,
    /// Declare the link dead after this much silence.
    pub dead_after: Duration,
}

impl ClientConfig {
    /// Defaults for the real app.
    pub fn new(endpoint: impl Fn() -> Option<Endpoint> + Send + 'static) -> Self {
        Self {
            endpoint: Box::new(endpoint),
            min_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(2),
            ping_after: Duration::from_secs(1),
            quiet_after: Duration::from_secs(2),
            dead_after: Duration::from_secs(3),
        }
    }
}
