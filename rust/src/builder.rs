use config;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::provider::OptionsProvider;

pub struct OptionsProviderBuilder {}

impl OptionsProviderBuilder {
    pub fn new() -> Self {
        OptionsProviderBuilder {}
    }

    pub fn add_directory(mut self, directory: PathBuf) -> Result<Self, String> {
        for entry in WalkDir::new(directory) {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                let path = entry.path();
                let file = config::File::from(path);
            }
        }
        return Ok(self);
    }

    pub fn build(self) -> OptionsProvider {
        OptionsProvider::new()
    }
}
