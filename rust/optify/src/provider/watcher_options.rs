use std::time::Duration;

/// Configuration options for the `OptionsWatcher`.
#[derive(Debug, Clone, Copy)]
pub struct WatcherOptions {
    /// The duration to wait before triggering a rebuild after file changes.
    pub debounce_duration: Duration,
}

impl Default for WatcherOptions {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_secs(1),
        }
    }
}

impl WatcherOptions {
    #[must_use]
    pub fn new(debounce_duration: Duration) -> Self {
        Self { debounce_duration }
    }
}
