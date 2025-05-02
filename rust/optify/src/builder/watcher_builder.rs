use std::path::{Path, PathBuf};

use crate::provider::OptionsWatcher;

use super::OptionsRegistryBuilder;

/// The duration to wait before triggering a rebuild after file changes.
pub const DEFAULT_DEBOUNCE_TIMEOUT_DURATION: std::time::Duration =
    std::time::Duration::from_secs(1);

/// A builder to use for local development to create an `OptionsWatcher` which changes the underlying `OptionsProvider` when files are changed.
///
/// This builder is kept separate from the `OptionsProviderBuilder` in order to keep `OptionsProviderBuilder` and `OptionsProvider` as simple and efficient as possible for production use.
///
/// ⚠️ Development in progress ⚠️\
/// Not truly considered public yet and mainly available to support bindings for other languages.
#[derive(Clone)]
pub struct OptionsWatcherBuilder {
    watched_directories: Vec<PathBuf>,
    debounce_timeout: std::time::Duration,
}

impl Default for OptionsWatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionsWatcherBuilder {
    pub fn new() -> Self {
        OptionsWatcherBuilder {
            watched_directories: Vec::new(),
            debounce_timeout: DEFAULT_DEBOUNCE_TIMEOUT_DURATION,
        }
    }

    pub fn with_debounce_timeout(
        &mut self,
        debounce_timeout: std::time::Duration,
    ) -> Result<&mut Self, String> {
        self.debounce_timeout = debounce_timeout;
        Ok(self)
    }
}

impl OptionsRegistryBuilder<OptionsWatcher> for OptionsWatcherBuilder {
    /// Add a directory to watch for changes.
    fn add_directory(&mut self, directory: &Path) -> Result<&Self, String> {
        self.watched_directories.push(directory.to_path_buf());
        Ok(self)
    }

    fn build(&mut self) -> Result<OptionsWatcher, String> {
        Ok(OptionsWatcher::new(
            self.watched_directories.clone(),
            self.debounce_timeout,
        ))
    }
}
