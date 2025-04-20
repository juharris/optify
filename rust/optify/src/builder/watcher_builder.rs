use std::path::{Path, PathBuf};

use crate::provider::OptionsWatcher;

pub struct OptionsWatcherBuilder {
    watched_directories: Vec<PathBuf>,
}

impl Default for OptionsWatcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionsWatcherBuilder {
    pub fn new() -> Self {
        Self {
            watched_directories: Vec::new(),
        }
    }

    pub fn add_directory(&mut self, directory: &Path) -> Result<&Self, String> {
        self.watched_directories.push(directory.to_path_buf());
        Ok(self)
    }

    pub fn build(&mut self) -> Result<OptionsWatcher, String> {
        Ok(OptionsWatcher::new(self.watched_directories.clone()))
    }
}
