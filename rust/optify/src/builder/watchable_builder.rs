use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{mpsc::channel, Arc, RwLock};

use crate::builder::{OptionsProviderBuilder, OptionsRegistryBuilder};
use crate::provider::{
    CacheOptions, Features, GetOptionsPreferences, OptionsProvider, OptionsRegistry,
};
use crate::schema::metadata::OptionsMetadata;

pub struct WatchableOptionsProviderBuilder {
    watched_directories: Vec<PathBuf>,
}

impl Default for WatchableOptionsProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WatchableOptionsProviderBuilder {
    pub fn new() -> Self {
        Self {
            watched_directories: Vec::new(),
        }
    }

    pub fn add_directory(&mut self, directory: &Path) -> Result<&Self, String> {
        self.watched_directories.push(directory.to_path_buf());
        Ok(self)
    }

    pub fn build(&mut self) -> Result<WatchableOptionsProvider, String> {
        Ok(WatchableOptionsProvider::new(
            self.watched_directories.clone(),
        ))
    }
}

pub struct WatchableOptionsProvider {
    current_provider: Arc<RwLock<OptionsProvider>>,
    watched_directories: Vec<PathBuf>,
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
}

impl WatchableOptionsProvider {
    fn new(watched_directories: Vec<PathBuf>) -> Self {
        let (tx, rx) = channel();
        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if let EventKind::Modify(_) = event.kind {
                        tx.send(event.paths).unwrap();
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            })
            .unwrap();

        // Set up initial watches first
        for dir in &watched_directories {
            watcher.watch(dir, RecursiveMode::Recursive).unwrap();
        }

        // Then build the initial provider
        let mut builder = OptionsProviderBuilder::new();
        for dir in &watched_directories {
            builder.add_directory(dir).unwrap();
        }
        let provider = builder.build().unwrap();

        let self_ = Self {
            current_provider: Arc::new(RwLock::new(provider)),
            watched_directories,
            watcher,
        };

        // Spawn a thread to handle file changes
        let current_provider = self_.current_provider.clone();
        let watched_directories = self_.watched_directories.clone();
        std::thread::spawn(move || {
            for _paths in rx {
                // When files change, rebuild the provider
                let mut builder = OptionsProviderBuilder::new();
                for dir in &watched_directories {
                    if let Err(e) = builder.add_directory(dir) {
                        println!("Error rebuilding provider: {}", e);
                        continue;
                    }
                }
                if let Ok(new_provider) = builder.build() {
                    // Atomically swap the provider
                    if let Ok(mut provider) = current_provider.write() {
                        *provider = new_provider;
                    }
                }
            }
        });

        self_
    }
}

impl OptionsRegistry for WatchableOptionsProvider {
    fn get_all_options(
        &self,
        feature_names: &[&str],
        cache_options: &Option<CacheOptions>,
        preferences: &Option<GetOptionsPreferences>,
    ) -> Result<serde_json::Value, String> {
        let provider = self.current_provider.read().unwrap();
        provider.get_all_options(feature_names, cache_options, preferences)
    }

    fn get_canonical_feature_name(&self, feature_name: &str) -> Result<String, String> {
        let provider = self.current_provider.read().unwrap();
        provider.get_canonical_feature_name(feature_name)
    }

    fn get_canonical_feature_names(&self, feature_names: &[&str]) -> Result<Vec<String>, String> {
        let provider = self.current_provider.read().unwrap();
        provider.get_canonical_feature_names(feature_names)
    }

    fn get_feature_metadata(&self, canonical_feature_name: &str) -> Option<OptionsMetadata> {
        let provider = self.current_provider.read().unwrap();
        provider.get_feature_metadata(canonical_feature_name)
    }

    fn get_features(&self) -> Vec<String> {
        let provider = self.current_provider.read().unwrap();
        provider.get_features()
    }

    fn get_features_with_metadata(&self) -> Features {
        let provider = self.current_provider.read().unwrap();
        provider.get_features_with_metadata()
    }

    fn get_options(&self, key: &str, feature_names: &[&str]) -> Result<serde_json::Value, String> {
        let provider = self.current_provider.read().unwrap();
        provider.get_options(key, feature_names)
    }

    fn get_options_with_preferences(
        &self,
        key: &str,
        feature_names: &[&str],
        cache_options: &Option<CacheOptions>,
        preferences: &Option<GetOptionsPreferences>,
    ) -> Result<serde_json::Value, String> {
        let provider = self.current_provider.read().unwrap();
        provider.get_options_with_preferences(key, feature_names, cache_options, preferences)
    }
}
