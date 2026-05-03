use optify::provider::{OptionsRegistry, OptionsWatcher};
use rustler::{Env, NifResult, ResourceArc, Term};
use std::sync::Mutex;

use crate::json_value_to_term;
use crate::preferences::PreferencesResource;

pub struct WatcherResource(pub Mutex<OptionsWatcher>);

#[rustler::resource_impl]
impl rustler::Resource for WatcherResource {}

// ===== Factory =====

#[rustler::nif]
pub fn watcher_build(directory: String) -> NifResult<ResourceArc<WatcherResource>> {
    match OptionsWatcher::build(&directory) {
        Ok(watcher) => Ok(ResourceArc::new(WatcherResource(Mutex::new(watcher)))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn watcher_build_with_schema(
    directory: String,
    schema_path: String,
) -> NifResult<ResourceArc<WatcherResource>> {
    match OptionsWatcher::build_with_schema(&directory, &schema_path) {
        Ok(watcher) => Ok(ResourceArc::new(WatcherResource(Mutex::new(watcher)))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn watcher_build_from_directories(
    directories: Vec<String>,
) -> NifResult<ResourceArc<WatcherResource>> {
    match OptionsWatcher::build_from_directories(&directories) {
        Ok(watcher) => Ok(ResourceArc::new(WatcherResource(Mutex::new(watcher)))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn watcher_build_from_directories_with_schema(
    directories: Vec<String>,
    schema_path: String,
) -> NifResult<ResourceArc<WatcherResource>> {
    match OptionsWatcher::build_from_directories_with_schema(&directories, &schema_path) {
        Ok(watcher) => Ok(ResourceArc::new(WatcherResource(Mutex::new(watcher)))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

// ===== Queries =====

#[rustler::nif]
pub fn watcher_get_features(watcher: ResourceArc<WatcherResource>) -> NifResult<Vec<String>> {
    let guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard.get_features())
}

#[rustler::nif]
pub fn watcher_get_aliases(watcher: ResourceArc<WatcherResource>) -> NifResult<Vec<String>> {
    let guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard.get_aliases())
}

#[rustler::nif]
pub fn watcher_get_features_and_aliases(
    watcher: ResourceArc<WatcherResource>,
) -> NifResult<Vec<String>> {
    let guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard.get_features_and_aliases())
}

#[rustler::nif]
pub fn watcher_get_canonical_feature_name(
    watcher: ResourceArc<WatcherResource>,
    feature_name: String,
) -> NifResult<String> {
    let guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard
        .get_canonical_feature_name(&feature_name)
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

#[rustler::nif]
pub fn watcher_get_canonical_feature_names(
    watcher: ResourceArc<WatcherResource>,
    feature_names: Vec<String>,
) -> NifResult<Vec<String>> {
    let guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard
        .get_canonical_feature_names(&feature_names)
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

// ===== Options =====

#[rustler::nif]
pub fn watcher_get_all_options<'a>(
    env: Env<'a>,
    watcher: ResourceArc<WatcherResource>,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Term<'a>> {
    let watch_guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    match watch_guard.get_all_options(&feature_names, None, Some(&prefs)) {
        Ok(options) => Ok(json_value_to_term(env, &options)),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn watcher_get_options<'a>(
    env: Env<'a>,
    watcher: ResourceArc<WatcherResource>,
    key: String,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Term<'a>> {
    let watch_guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    match watch_guard.get_options_with_preferences(&key, &feature_names, None, Some(&prefs)) {
        Ok(options) => Ok(json_value_to_term(env, &options)),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn watcher_get_filtered_feature_names(
    watcher: ResourceArc<WatcherResource>,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Vec<String>> {
    let watch_guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    watch_guard
        .get_filtered_feature_names(&feature_names, Some(&prefs))
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

#[rustler::nif]
pub fn watcher_map_feature_names(
    watcher: ResourceArc<WatcherResource>,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Vec<Option<String>>> {
    let watch_guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    watch_guard
        .map_feature_names(&feature_names, Some(&prefs))
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

#[rustler::nif]
pub fn watcher_has_conditions(
    watcher: ResourceArc<WatcherResource>,
    canonical_feature_name: String,
) -> NifResult<bool> {
    let guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard.has_conditions(&canonical_feature_name))
}

#[rustler::nif]
pub fn watcher_last_modified(watcher: ResourceArc<WatcherResource>) -> NifResult<u64> {
    let guard = watcher
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard
        .last_modified()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
}
