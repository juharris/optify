use notify_debouncer_full::{new_debouncer, notify::RecommendedWatcher, DebounceEventResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{mpsc::channel, Arc, Mutex, RwLock};

use crate::builder::{OptionsProviderBuilder, OptionsRegistryBuilder, OptionsWatcherBuilder};
use crate::provider::{
    CacheOptions, Features, GetOptionsPreferences, OptionsProvider, OptionsRegistry,
};
use crate::schema::metadata::OptionsMetadata;

/// The duration to wait before triggering a rebuild after file changes.
pub const DEFAULT_DEBOUNCE_DURATION: std::time::Duration = std::time::Duration::from_secs(1);

/// A registry which changes the underlying when files are changed.
/// This is mainly meant to use for local development.
///
/// ⚠️ Development in progress ⚠️\
/// Not truly considered public yet and mainly available to support bindings for other languages.
pub struct OptionsWatcher {
    current_provider: Arc<RwLock<OptionsProvider>>,
    last_modified: Arc<Mutex<std::time::SystemTime>>,
    watched_directories: Vec<PathBuf>,
    // The watcher needs to be held to continue watching files for changes.
    #[allow(dead_code)]
    debouncer_watcher: notify_debouncer_full::Debouncer<
        RecommendedWatcher,
        notify_debouncer_full::RecommendedCache,
    >,
}

impl OptionsWatcher {
    pub(crate) fn new(watched_directories: Vec<PathBuf>) -> Self {
        // Set up the watcher before building in case the files change before building.
        let (tx, rx) = channel();
        let mut debouncer_watcher = new_debouncer(
            DEFAULT_DEBOUNCE_DURATION,
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    let paths = events
                        .iter()
                        .filter(|event| !event.kind.is_access())
                        .filter(|event| {
                            // Ignore metadata changes such as the modified time.
                            match event.kind {
                                notify::EventKind::Modify(modify_kind) => {
                                    !matches!(modify_kind, notify::event::ModifyKind::Metadata(_))
                                }
                                _ => true,
                            }
                        })
                        .flat_map(|event| event.paths.clone())
                        .collect::<HashSet<_>>();

                    if paths.is_empty() {
                        return;
                    }

                    eprintln!(
                        "[optify] Rebuilding OptionsProvider because contents at these path(s) changed: {paths:?}"
                    );

                    tx.send(()).unwrap();
                }
                Err(errors) => errors
                    .iter()
                    .for_each(|error| eprintln!("\x1b[31m[optify] {error:?}\x1b[0m")),
            },
        )
        .unwrap();
        for dir in &watched_directories {
            debouncer_watcher
                .watch(dir, notify::RecursiveMode::Recursive)
                .expect("directory to be watched");
        }
        let mut builder = OptionsProviderBuilder::new();
        for dir in &watched_directories {
            builder
                .add_directory(dir)
                .expect("directory and contents to be valid");
        }
        let provider = builder.build().expect("provider to be built");
        let last_modified = Arc::new(Mutex::new(std::time::SystemTime::now()));

        let self_ = Self {
            current_provider: Arc::new(RwLock::new(provider)),
            last_modified,
            watched_directories,
            debouncer_watcher,
        };

        let current_provider = self_.current_provider.clone();
        let watched_directories = self_.watched_directories.clone();
        let last_modified = self_.last_modified.clone();

        std::thread::spawn(move || {
            for _ in rx {
                let result = std::panic::catch_unwind(|| {
                    let mut skip_rebuild = false;
                    let mut builder = OptionsProviderBuilder::new();
                    for dir in &watched_directories {
                        if dir.exists() {
                            if let Err(e) = builder.add_directory(dir) {
                                eprintln!("\x1b[31m[optify] Error rebuilding provider: {e}\x1b[0m");
                                skip_rebuild = true;
                                break;
                            }
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
                                *last_modified.lock().unwrap() = std::time::SystemTime::now();
                                eprintln!("\x1b[32m[optify] Successfully rebuilt the OptionsProvider.\x1b[0m");
                            }
                            Err(err) => {
                                eprintln!(
                                    "\x1b[31m[optify] Error rebuilding provider: {err}\nWill not change the provider until the files are fixed.\x1b[0m"
                                );
                            }
                        },
                        Err(err) => {
                            eprintln!("\x1b[31m[optify] Error rebuilding provider: {err}\x1b[0m");
                        }
                    }
                });

                if result.is_err() {
                    eprintln!("\x1b[31m[optify] Error rebuilding the provider. Will not change the provider until the files are fixed.\x1b[0m");
                }
            }
        });

        self_
    }

    pub fn build(directory: &Path) -> Result<OptionsWatcher, String> {
        let mut builder = OptionsWatcherBuilder::new();
        builder.add_directory(directory)?;
        builder.build()
    }

    pub fn build_from_directories(
        directories: &[impl AsRef<Path>],
    ) -> Result<OptionsWatcher, String> {
        let mut builder = OptionsWatcherBuilder::new();
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
        builder.build()
    }

    /// Returns the time when the provider was finished building.
    pub fn last_modified(&self) -> std::time::SystemTime {
        *self.last_modified.lock().unwrap()
    }
}

impl OptionsRegistry for OptionsWatcher {
    fn get_aliases(&self) -> Vec<String> {
        self.current_provider.read().unwrap().get_aliases()
    }

    fn get_features_and_aliases(&self) -> Vec<String> {
        self.current_provider
            .read()
            .unwrap()
            .get_features_and_aliases()
    }

    fn get_all_options(
        &self,
        feature_names: &[impl AsRef<str>],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> std::result::Result<serde_json::Value, String> {
        self.current_provider.read().unwrap().get_all_options(
            feature_names,
            cache_options,
            preferences,
        )
    }

    fn get_canonical_feature_name(
        &self,
        feature_name: &str,
    ) -> std::result::Result<String, String> {
        self.current_provider
            .read()
            .unwrap()
            .get_canonical_feature_name(feature_name)
    }

    fn get_canonical_feature_names(
        &self,
        feature_names: &[impl AsRef<str>],
    ) -> std::result::Result<Vec<String>, String> {
        self.current_provider
            .read()
            .unwrap()
            .get_canonical_feature_names(feature_names)
    }

    fn get_feature_metadata(&self, canonical_feature_name: &str) -> Option<OptionsMetadata> {
        self.current_provider
            .read()
            .unwrap()
            .get_feature_metadata(canonical_feature_name)
    }

    fn get_features(&self) -> Vec<String> {
        self.current_provider.read().unwrap().get_features()
    }

    fn get_features_with_metadata(&self) -> Features {
        self.current_provider
            .read()
            .unwrap()
            .get_features_with_metadata()
    }

    fn get_options(
        &self,
        key: &str,
        feature_names: &[impl AsRef<str>],
    ) -> std::result::Result<serde_json::Value, String> {
        self.current_provider
            .read()
            .unwrap()
            .get_options(key, feature_names)
    }

    fn get_options_with_preferences(
        &self,
        key: &str,
        feature_names: &[impl AsRef<str>],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> std::result::Result<serde_json::Value, String> {
        self.current_provider
            .read()
            .unwrap()
            .get_options_with_preferences(key, feature_names, cache_options, preferences)
    }
}
