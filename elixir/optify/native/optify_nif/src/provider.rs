use optify::provider::{OptionsProvider, OptionsRegistry};
use rustler::{Env, NifResult, ResourceArc, Term};

use crate::json_value_to_term;
use crate::preferences::PreferencesResource;

pub struct ProviderResource(pub OptionsProvider);

#[rustler::resource_impl]
impl rustler::Resource for ProviderResource {}

// ===== Factory =====

#[rustler::nif(schedule = "DirtyCpu")]
pub fn provider_build(directory: String) -> NifResult<ResourceArc<ProviderResource>> {
    match OptionsProvider::build(&directory) {
        Ok(provider) => Ok(ResourceArc::new(ProviderResource(provider))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn provider_build_with_schema(
    directory: String,
    schema_path: String,
) -> NifResult<ResourceArc<ProviderResource>> {
    match OptionsProvider::build_with_schema(&directory, &schema_path) {
        Ok(provider) => Ok(ResourceArc::new(ProviderResource(provider))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn provider_build_from_directories(
    directories: Vec<String>,
) -> NifResult<ResourceArc<ProviderResource>> {
    match OptionsProvider::build_from_directories(&directories) {
        Ok(provider) => Ok(ResourceArc::new(ProviderResource(provider))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
pub fn provider_build_from_directories_with_schema(
    directories: Vec<String>,
    schema_path: String,
) -> NifResult<ResourceArc<ProviderResource>> {
    match OptionsProvider::build_from_directories_with_schema(&directories, &schema_path) {
        Ok(provider) => Ok(ResourceArc::new(ProviderResource(provider))),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

// ===== Queries =====

#[rustler::nif]
pub fn provider_get_features(provider: ResourceArc<ProviderResource>) -> Vec<String> {
    provider.0.get_features()
}

#[rustler::nif]
pub fn provider_get_aliases(provider: ResourceArc<ProviderResource>) -> Vec<String> {
    provider.0.get_aliases()
}

#[rustler::nif]
pub fn provider_get_features_and_aliases(provider: ResourceArc<ProviderResource>) -> Vec<String> {
    provider.0.get_features_and_aliases()
}

#[rustler::nif]
pub fn provider_get_canonical_feature_name(
    provider: ResourceArc<ProviderResource>,
    feature_name: String,
) -> NifResult<String> {
    provider
        .0
        .get_canonical_feature_name(&feature_name)
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

#[rustler::nif]
pub fn provider_get_canonical_feature_names(
    provider: ResourceArc<ProviderResource>,
    feature_names: Vec<String>,
) -> NifResult<Vec<String>> {
    provider
        .0
        .get_canonical_feature_names(&feature_names)
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

// ===== Options =====

#[rustler::nif]
pub fn provider_get_all_options<'a>(
    env: Env<'a>,
    provider: ResourceArc<ProviderResource>,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Term<'a>> {
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    match provider
        .0
        .get_all_options(&feature_names, None, Some(&prefs))
    {
        Ok(options) => Ok(json_value_to_term(env, &options)),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn provider_get_options<'a>(
    env: Env<'a>,
    provider: ResourceArc<ProviderResource>,
    key: String,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Term<'a>> {
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    match provider
        .0
        .get_options_with_preferences(&key, &feature_names, None, Some(&prefs))
    {
        Ok(options) => Ok(json_value_to_term(env, &options)),
        Err(e) => Err(rustler::Error::Term(Box::new(e))),
    }
}

#[rustler::nif]
pub fn provider_get_filtered_feature_names(
    provider: ResourceArc<ProviderResource>,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Vec<String>> {
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    provider
        .0
        .get_filtered_feature_names(&feature_names, Some(&prefs))
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

#[rustler::nif]
pub fn provider_map_feature_names(
    provider: ResourceArc<ProviderResource>,
    feature_names: Vec<String>,
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Vec<Option<String>>> {
    let prefs = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    provider
        .0
        .map_feature_names(&feature_names, Some(&prefs))
        .map_err(|e| rustler::Error::Term(Box::new(e)))
}

#[rustler::nif]
pub fn provider_has_conditions(
    provider: ResourceArc<ProviderResource>,
    canonical_feature_name: String,
) -> bool {
    provider.0.has_conditions(&canonical_feature_name)
}
