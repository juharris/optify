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

// TODO Move to another file.
pub struct WatchableOptionsProvider {
    current_provider: Arc<RwLock<OptionsProvider>>,
    watched_directories: Vec<PathBuf>,
    // The watcher needs to be held to continue watching files for changes.
    #[allow(dead_code)]
    watcher: notify::RecommendedWatcher,
}

impl WatchableOptionsProvider {
    fn new(watched_directories: Vec<PathBuf>) -> Self {
        // Set up the watcher before building in case the files change before building.
        let (tx, rx) = channel();
        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    // Seems like no other types of events are needed as the tests show that we can handle a few cases, but let's add more tests, like adding a subdirectory.
                    if let EventKind::Modify(_) = event.kind {
                        tx.send(event.paths).unwrap();
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            })
            .unwrap();

        for dir in &watched_directories {
            watcher.watch(dir, RecursiveMode::Recursive).unwrap();
        }

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

        let current_provider = self_.current_provider.clone();
        let watched_directories = self_.watched_directories.clone();
        std::thread::spawn(move || {
            for _paths in rx {
                println!(
                    "[optify] Rebuilding OptionsProvider because contents at these path(s) changed: {:?}",
                    _paths
                );
                let result = std::panic::catch_unwind(|| {
                    let mut skip_rebuild = false;
                    let mut builder = OptionsProviderBuilder::new();
                    for dir in &watched_directories {
                        if let Err(e) = builder.add_directory(dir) {
                            println!("\x1b[31m[optify] Error rebuilding provider: {}\x1b[0m", e);
                            skip_rebuild = true;
                            break;
                        }
                    }

                    if skip_rebuild {
                        // Ignore errors because the developer might still be changing the files.
                        // TODO If there are still errors after a few minutes, then consider panicking.
                        return;
                    }

                    match builder.build() {
                        Ok(new_provider) => match current_provider.write() {
                            Ok(mut provider) => {
                                *provider = new_provider;
                                println!("\x1b[32m[optify] Successfully rebuilt the OptionsProvider.\x1b[0m");
                            }
                            Err(err) => {
                                println!(
                                    "\x1b[31m[optify] Error rebuilding provider: {}\nWill not change the provider until the files are fixed.\x1b[0m",
                                    err
                                );
                            }
                        },
                        Err(err) => {
                            println!("\x1b[31m[optify] Error rebuilding provider: {}\x1b[0m", err);
                        }
                    }
                });

                if result.is_err() {
                    println!("\x1b[31m[optify] Error rebuilding the provider. Will not change the provider until the files are fixed.\x1b[0m");
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
