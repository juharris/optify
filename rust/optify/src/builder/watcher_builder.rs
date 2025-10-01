use std::path::{Path, PathBuf};

use crate::provider::{OptionsWatcher, WatcherOptions};

use super::OptionsRegistryBuilder;

/// A builder to use for local development to create an `OptionsWatcher` which changes the underlying `OptionsProvider` when files are changed.
///
/// This builder is kept separate from the `OptionsProviderBuilder` in order to keep `OptionsProviderBuilder` and `OptionsProvider` as simple and efficient as possible for production use.
///
/// ⚠️ Development in progress ⚠️\
/// Not truly considered public yet and mainly available to support bindings for other languages.
#[derive(Clone)]
pub struct OptionsWatcherBuilder {
    schema_path: Option<PathBuf>,
    watched_directories: Vec<PathBuf>,
    watcher_options: WatcherOptions,
}

impl Default for OptionsWatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionsWatcherBuilder {
    pub fn new() -> Self {
        OptionsWatcherBuilder {
            schema_path: None,
            watched_directories: Vec::new(),
            watcher_options: WatcherOptions::default(),
        }
    }

    pub fn with_watcher_options(&mut self, watcher_options: WatcherOptions) -> &mut Self {
        self.watcher_options = watcher_options;
        self
    }
}

impl OptionsRegistryBuilder<OptionsWatcher> for OptionsWatcherBuilder {
    /// Add a directory to watch for changes.
    fn add_directory(&mut self, directory: impl AsRef<Path>) -> Result<&Self, String> {
        self.watched_directories
            .push(directory.as_ref().to_path_buf());
        Ok(self)
    }

    fn with_schema(&mut self, schema_path: impl AsRef<Path>) -> Result<&Self, String> {
        self.schema_path = Some(schema_path.as_ref().to_path_buf());
        Ok(self)
    }

    fn build(&mut self) -> Result<OptionsWatcher, String> {
        OptionsWatcher::new(
            &self.watched_directories,
            self.schema_path.as_ref(),
            self.watcher_options.clone(),
        )
    }
}
