use notify_debouncer_full::{new_debouncer, notify::RecommendedWatcher, DebounceEventResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{mpsc::channel, Arc, Mutex, RwLock};

use crate::builder::{OptionsProviderBuilder, OptionsRegistryBuilder, OptionsWatcherBuilder};
use crate::provider::{
    CacheOptions, Features, GetOptionsPreferences, OptionsProvider, OptionsRegistry, WatcherOptions,
};
use crate::schema::metadata::OptionsMetadata;

pub type OptionsWatcherListener = Arc<dyn Fn(&HashSet<PathBuf>) + Send + Sync>;

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
    listeners: Arc<Mutex<Vec<OptionsWatcherListener>>>,
}

impl OptionsWatcher {
    pub fn build_with_options(
        directory: impl AsRef<Path>,
        watcher_options: WatcherOptions,
    ) -> Result<Self, String> {
        let mut builder = OptionsWatcherBuilder::new();
        builder.with_watcher_options(watcher_options);
        builder.add_directory(directory.as_ref())?;
        builder.build()
    }

    pub fn build_with_schema_and_options(
        directory: impl AsRef<Path>,
        schema_path: impl AsRef<Path>,
        watcher_options: WatcherOptions,
    ) -> Result<Self, String> {
        let mut builder = OptionsWatcherBuilder::new();
        builder.with_schema(schema_path.as_ref())?;
        builder.add_directory(directory.as_ref())?;
        builder.with_watcher_options(watcher_options);
        builder.build()
    }

    pub fn build_from_directories_with_options(
        directories: &[impl AsRef<Path>],
        watcher_options: WatcherOptions,
    ) -> Result<Self, String> {
        let mut builder = OptionsWatcherBuilder::new();
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
        builder.with_watcher_options(watcher_options);
        builder.build()
    }

    pub fn build_from_directories_with_schema_and_options(
        directories: &[impl AsRef<Path>],
        schema_path: impl AsRef<Path>,
        watcher_options: WatcherOptions,
    ) -> Result<Self, String> {
        let mut builder = OptionsWatcherBuilder::new();
        builder.with_watcher_options(watcher_options);
        builder.with_schema(schema_path.as_ref())?;
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
        builder.build()
    }

    pub(crate) fn new(
        watched_directories: &[impl AsRef<Path>],
        schema_path: Option<impl AsRef<Path>>,
        watcher_options: WatcherOptions,
    ) -> Result<Self, String> {
        // Set up the watcher before building in case the files change before building.
        let (tx, rx) = channel();
        let mut debouncer_watcher = new_debouncer(
            watcher_options.debounce_duration,
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

                    tx.send(paths).unwrap();
                }
                Err(errors) => errors
                    .iter()
                    .for_each(|error| eprintln!("\x1b[31m[optify] {error:?}\x1b[0m")),
            },
        )
        .map_err(|e| format!("Failed to create debouncer: {e}"))?;
        for dir in watched_directories {
            debouncer_watcher
                .watch(dir, notify::RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch directory {:?}: {e}", dir.as_ref()))?;
        }
        let mut builder = OptionsProviderBuilder::new();
        if let Some(schema) = schema_path {
            builder
                .with_schema(&schema)
                .map_err(|e| format!("Invalid schema: {e}"))?;
        }
        for dir in watched_directories {
            builder
                .add_directory(dir)
                .map_err(|e| format!("Failed to add directory {:?}: {e}", dir.as_ref()))?;
        }
        let provider = builder
            .build_and_clear()
            .map_err(|e| format!("Failed to build provider: {e}"))?;
        let last_modified = Arc::new(Mutex::new(std::time::SystemTime::now()));

        let self_ = Self {
            current_provider: Arc::new(RwLock::new(provider)),
            last_modified,
            watched_directories: watched_directories
                .iter()
                .map(|dir| dir.as_ref().to_path_buf())
                .collect(),
            debouncer_watcher,
            listeners: Arc::new(Mutex::new(Vec::new())),
        };

        let current_provider = self_.current_provider.clone();
        let watched_directories = self_.watched_directories.clone();
        let last_modified = self_.last_modified.clone();
        let listeners = self_.listeners.clone();

        std::thread::spawn(move || {
            for paths in rx {
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

                    match builder.build_and_clear() {
                        Ok(new_provider) => match current_provider.write() {
                            Ok(mut provider) => {
                                *provider = new_provider;
                                *last_modified.lock().unwrap() = std::time::SystemTime::now();
                                eprintln!("\x1b[32m[optify] Successfully rebuilt the OptionsProvider.\x1b[0m");
                                let listeners_guard = listeners.lock().unwrap();
                                for listener in listeners_guard.iter() {
                                    listener(&paths);
                                }
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

        Ok(self_)
    }

    pub fn add_listener(&mut self, listener: OptionsWatcherListener) {
        self.listeners.lock().unwrap().push(listener);
    }

    /// Returns the time when the provider was finished building.
    pub fn last_modified(&self) -> std::time::SystemTime {
        *self.last_modified.lock().unwrap()
    }
}

impl OptionsRegistry for OptionsWatcher {
    fn build(directory: impl AsRef<Path>) -> Result<OptionsWatcher, String> {
        let mut builder = OptionsWatcherBuilder::new();
        builder.add_directory(directory.as_ref())?;
        builder.build()
    }

    fn build_with_schema(
        directory: impl AsRef<Path>,
        schema_path: impl AsRef<Path>,
    ) -> Result<OptionsWatcher, String> {
        let mut builder = OptionsWatcherBuilder::new();
        builder.with_schema(schema_path.as_ref())?;
        builder.add_directory(directory.as_ref())?;
        builder.build()
    }

    fn build_from_directories(directories: &[impl AsRef<Path>]) -> Result<OptionsWatcher, String> {
        let mut builder = OptionsWatcherBuilder::new();
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
        builder.build()
    }

    fn build_from_directories_with_schema(
        directories: &[impl AsRef<Path>],
        schema_path: impl AsRef<Path>,
    ) -> Result<OptionsWatcher, String> {
        let mut builder = OptionsWatcherBuilder::new();
        builder.with_schema(schema_path.as_ref())?;
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
        builder.build()
    }

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

    fn get_filtered_feature_names(
        &self,
        feature_names: &[impl AsRef<str>],
        preferences: Option<&GetOptionsPreferences>,
    ) -> std::result::Result<Vec<String>, String> {
        self.current_provider
            .read()
            .unwrap()
            .get_filtered_feature_names(feature_names, preferences)
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

    fn has_conditions(&self, canonical_feature_name: &str) -> bool {
        self.current_provider
            .read()
            .unwrap()
            .has_conditions(canonical_feature_name)
    }
}
