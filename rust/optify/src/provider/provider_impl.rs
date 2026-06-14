use std::{collections::HashMap, path::Path, sync::RwLock};

use crate::builder::builder_options::BuilderOptions;
use crate::configurable_values::configurable_list_impl::ConfigurableList;

use crate::{
    builder::{OptionsProviderBuilder, OptionsRegistryBuilder},
    configurable_string::LoadedFiles,
    json::merge::merge_json_with_defaults,
    provider::GetOptionsPreferences,
    schema::{conditions::ConditionExpression, metadata::OptionsMetadata},
};

use super::OptionsRegistry;
use crate::configurable_string::ConfigurableString;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

pub(crate) type SourceValue = serde_json::Value;

pub(crate) type Aliases = HashMap<unicase::UniCase<String>, String>;
pub(crate) type Conditions = HashMap<String, ConditionExpression>;
pub(crate) type Features = HashMap<String, OptionsMetadata>;
pub(crate) type ReferencedFileToFeatureNames = HashMap<String, Vec<String>>;
pub(crate) type Sources = HashMap<String, SourceValue>;

pub(crate) type EntireConfigCache = HashMap<Vec<String>, serde_json::Value>;
pub(crate) type OptionsCache = HashMap<(String, Vec<String>, bool), serde_json::Value>;

pub struct CacheOptions {}

pub struct OptionsProvider {
    // Configurable Values
    all_configurable_list_pointers: Vec<String>,
    all_configurable_string_pointers: Vec<String>,
    keyed_configurable_list_pointers: HashMap<String, Vec<String>>,
    keyed_configurable_string_pointers: HashMap<String, Vec<String>>,

    aliases: Aliases,
    conditions: Conditions,
    features: Features,
    /// A map of files to their referencing features.
    /// The keys are relative file paths and the values are lists of canonical feature names.
    /// This allows fast lookup of features when a specific file is modified.
    /// This is only populated if the `BuilderOptions` enable file reference tracking.
    referenced_file_to_feature_names: Option<ReferencedFileToFeatureNames>,
    loaded_files: LoadedFiles,
    sources: Sources,

    // Caches - using RwLock for thread-safe interior mutability
    entire_config_cache: RwLock<EntireConfigCache>,
    options_cache: RwLock<OptionsCache>,
}

impl OptionsProvider {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        aliases: Aliases,
        all_configurable_list_pointers: Vec<String>,
        all_configurable_string_pointers: Vec<String>,
        keyed_configurable_list_pointers: HashMap<String, Vec<String>>,
        keyed_configurable_string_pointers: HashMap<String, Vec<String>>,
        conditions: Conditions,
        features: Features,
        referenced_file_to_feature_names: Option<ReferencedFileToFeatureNames>,
        loaded_files: LoadedFiles,
        sources: Sources,
    ) -> Self {
        OptionsProvider {
            all_configurable_list_pointers,
            all_configurable_string_pointers,
            keyed_configurable_list_pointers,
            keyed_configurable_string_pointers,
            aliases,
            conditions,
            features,
            referenced_file_to_feature_names,
            loaded_files,
            sources,
            entire_config_cache: RwLock::new(EntireConfigCache::new()),
            options_cache: RwLock::new(OptionsCache::new()),
        }
    }

    fn get_entire_config(
        &self,
        feature_names: &[String],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<serde_json::Value, String> {
        if cache_options.is_some() {
            match self.get_entire_config_from_cache(feature_names, preferences) {
                Ok(Some(config)) => return Ok(config),
                Ok(None) => (),
                Err(e) => return Err(e),
            }
        };

        let overrides = preferences.and_then(|p| p.overrides.as_ref());

        let result = match (overrides, feature_names.len()) {
            (None, 0) => serde_json::Value::Object(serde_json::Map::new()),
            (None, 1) => {
                // Avoid merging to an empty object and eagerly take the right configuration.
                let feature_name = &feature_names[0];
                self.sources
                    .get(feature_name)
                    .ok_or_else(|| {
                        format!("Feature name {feature_name:?} is not a known feature.")
                    })?
                    .clone()
            }
            (Some(overrides), 0) => overrides.clone(),
            _ => {
                // Start with overrides as base (highest priority), or last source if no overrides.
                // Sources are ordered from lowest to highest priority, so we iterate in reverse.
                let mut result = match overrides {
                    Some(overrides) => overrides.clone(),
                    None => {
                        let canonical_feature_name = feature_names.last().unwrap();
                        self.sources
                            .get(canonical_feature_name)
                            .ok_or_else(|| {
                                // Should not happen.
                                // All canonical feature names are included as keys in the sources map.
                                // It could happen in the future if we allow aliases to be added directly, but we should try to validate them when the provider is built.
                                format!("Feature name {canonical_feature_name:?} is not a known feature.")
                            })?
                            .clone()
                    }
                };

                // Merge sources as defaults in reverse (highest to lowest priority).
                // Skip last source if no overrides (already used as base).
                let skip_count = if overrides.is_some() { 0 } else { 1 };
                for canonical_feature_name in feature_names.iter().rev().skip(skip_count) {
                    let source = self.sources.get(canonical_feature_name).ok_or_else(|| {
                        // Should not happen.
                        // All canonical feature names are included as keys in the sources map.
                        // It could happen in the future if we allow aliases to be added directly, but we should try to validate them when the provider is built.
                        format!("Feature name {canonical_feature_name:?} is not a known feature.")
                    })?;
                    merge_json_with_defaults(&mut result, source);
                }

                result
            }
        };

        if cache_options.is_some() {
            let cache_key = feature_names.to_owned();
            self.entire_config_cache
                .write()
                .expect("the entire config cache lock should be held")
                .insert(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Get options for a specific key by extracting and merging only that key from each source.
    /// This avoids building the entire config when only one key is needed.
    fn get_options_for_key(
        &self,
        key: &str,
        filtered_feature_names: &[String],
        original_feature_names: &[impl AsRef<str>],
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<serde_json::Value, String> {
        let override_for_key = preferences
            .and_then(|p| p.overrides.as_ref())
            .and_then(|o| o.get(key));

        let result: Option<serde_json::Value> =
            match (override_for_key, filtered_feature_names.len()) {
                (None, 0) => None,
                (None, 1) => {
                    let feature_name = &filtered_feature_names[0];
                    let source = self.sources.get(feature_name).ok_or_else(|| {
                        format!("Feature name {feature_name:?} is not a known feature.")
                    })?;
                    source.get(key).cloned()
                }
                (None, _) => {
                    // No override. Find first source with key (iterating in reverse = highest priority first).
                    // Use the last source with the key to avoid merging to an empty object.
                    let mut result = None;
                    for canonical_feature_name in filtered_feature_names.iter().rev() {
                        let source = self.sources.get(canonical_feature_name).ok_or_else(|| {
                            format!(
                                "Feature name {canonical_feature_name:?} is not a known feature."
                            )
                        })?;

                        if let Some(source_value) = source.get(key) {
                            match &mut result {
                                Some(existing) => merge_json_with_defaults(existing, source_value),
                                None => result = Some(source_value.clone()),
                            }
                        }
                    }
                    result
                }
                (Some(override_value), 0) => Some(override_value.clone()),
                (Some(override_value), _) => {
                    // Start with override as base (highest priority).
                    // Sources are ordered from lowest to highest priority, so we iterate in reverse.
                    let mut result = override_value.clone();
                    for canonical_feature_name in filtered_feature_names.iter().rev() {
                        let source = self.sources.get(canonical_feature_name).ok_or_else(|| {
                            format!(
                                "Feature name {canonical_feature_name:?} is not a known feature."
                            )
                        })?;

                        if let Some(source_value) = source.get(key) {
                            merge_json_with_defaults(&mut result, source_value);
                        }
                    }
                    Some(result)
                }
            };

        result.ok_or_else(|| {
            format!(
                "Error getting options with features {:?}: configuration property \"{}\" not found",
                original_feature_names
                    .iter()
                    .map(|f| f.as_ref())
                    .collect::<Vec<&str>>(),
                key
            )
        })
    }

    fn get_entire_config_from_cache(
        &self,
        feature_names: &[String],
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Option<serde_json::Value>, String> {
        if let Some(preferences) = preferences {
            if preferences.overrides.is_some() {
                return Err("Caching when overrides are given is not supported.".to_owned());
            }
        }
        let cache_key = feature_names.to_owned();
        if let Some(config) = self
            .entire_config_cache
            .read()
            .expect("the entire config cache should be readable")
            .get(&cache_key)
        {
            return Ok(Some(config.clone()));
        }

        Ok(None)
    }

    pub fn get_options_from_cache(
        &self,
        key: &str,
        feature_names: &[impl AsRef<str>],
        _cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Option<serde_json::Value>, String> {
        let filtered_feature_names = self.get_filtered_feature_names(feature_names, preferences)?;
        let are_configurable_strings_enabled = preferences
            .map(|p| p.are_configurable_strings_enabled)
            .unwrap_or(false);
        let cache_key = (
            key.to_owned(),
            filtered_feature_names,
            are_configurable_strings_enabled,
        );
        if let Some(options) = self
            .options_cache
            .read()
            .expect("the options cache should be readable")
            .get(&cache_key)
        {
            return Ok(Some(options.clone()));
        }

        Ok(None)
    }

    fn process_configurable_lists(
        &self,
        value: &mut serde_json::Value,
        key: Option<&str>,
    ) -> Result<(), String> {
        match key {
            Some(key) => match self.keyed_configurable_list_pointers.get(key) {
                Some(pointers) => {
                    for pointer in pointers {
                        self.handle_configurable_list_pointer(value, pointer)?;
                    }
                }
                _ => {
                    // There are no pointers for the key.
                }
            },
            None => {
                // There is no key prefix when the entire configuration is requested.
                for pointer in &self.all_configurable_list_pointers {
                    self.handle_configurable_list_pointer(value, pointer)?;
                }
            }
        }

        Ok(())
    }

    fn handle_configurable_list_pointer(
        &self,
        value: &mut serde_json::Value,
        pointer: &String,
    ) -> Result<(), String> {
        if let Some(configurable_value) = value.pointer_mut(pointer) {
            // Only continue if it has the right indicator property because it may have been overridden.
            if let Some(type_value) =
                configurable_value.get(crate::configurable_values::locator::TYPE_KEY)
            {
                if let Some(type_str) = type_value.as_str() {
                    if type_str != crate::configurable_values::locator::LIST_TYPE {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }

            let configurable_list: ConfigurableList =
                match serde_json::from_value(configurable_value.clone()) {
                    Ok(cl) => cl,
                    Err(e) => {
                        return Err(format!(
                            "Failed to deserialize ConfigurableList at {}: {}",
                            pointer, e
                        ));
                    }
                };

            // Replace the value at the pointer location with the built list.
            let built_list = configurable_list.build()?;
            *configurable_value = serde_json::Value::Array(built_list);
        }

        Ok(())
    }

    /// Process configurable strings in the JSON value based on the pointers.
    fn process_configurable_strings(
        &self,
        value: &mut serde_json::Value,
        key: Option<&str>,
    ) -> Result<(), String> {
        match key {
            Some(key) => match self.keyed_configurable_string_pointers.get(key) {
                Some(pointers) => {
                    for pointer in pointers {
                        self.handle_configurable_string_pointer(value, pointer)?;
                    }
                }
                _ => {
                    // There are no pointers for the key.
                }
            },
            None => {
                // There is no key prefix when the entire configuration is requested.
                for pointer in &self.all_configurable_string_pointers {
                    self.handle_configurable_string_pointer(value, pointer)?;
                }
            }
        }

        Ok(())
    }

    fn handle_configurable_string_pointer(
        &self,
        value: &mut serde_json::Value,
        pointer: &String,
    ) -> Result<(), String> {
        if let Some(configurable_value) = value.pointer_mut(pointer) {
            // Only continue if it has the right indicator property because it may have been overridden.
            if let Some(type_value) =
                configurable_value.get(crate::configurable_values::locator::TYPE_KEY)
            {
                if let Some(type_str) = type_value.as_str() {
                    if type_str != crate::configurable_values::locator::STRING_TYPE {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }

            let configurable_string: ConfigurableString =
                match serde_json::from_value(configurable_value.clone()) {
                    Ok(cs) => cs,
                    Err(e) => {
                        return Err(format!(
                            "Failed to deserialize ConfigurableString at {}: {}",
                            pointer, e
                        ));
                    }
                };

            // Replace the value at the pointer location with the built string.
            let built_string = configurable_string.build(&self.loaded_files)?;
            *configurable_value = serde_json::Value::String(built_string);
        }
        Ok(())
    }
}

impl OptionsRegistry for OptionsProvider {
    fn build(directory: impl AsRef<Path>) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(directory.as_ref())?;
        builder.build_and_clear()
    }

    fn build_from_directories(directories: &[impl AsRef<Path>]) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directories(directories)?;
        builder.build_and_clear()
    }

    fn build_from_directories_with_options(
        directories: &[impl AsRef<Path>],
        options: BuilderOptions,
    ) -> Result<Self, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.with_options(options)?;
        builder.add_directories(directories)?;
        builder.build_and_clear()
    }

    fn build_with_options(
        directory: impl AsRef<Path>,
        options: BuilderOptions,
    ) -> Result<Self, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.with_options(options)?;
        builder.add_directory(directory.as_ref())?;
        builder.build_and_clear()
    }

    fn get_aliases(&self) -> Vec<String> {
        self.features
            .values()
            .filter_map(|metadata| metadata.aliases.as_ref())
            .flatten()
            .cloned()
            .collect()
    }

    fn get_features_and_aliases(&self) -> Vec<String> {
        self.aliases.keys().map(|k| k.to_string()).collect()
    }

    fn get_all_options(
        &self,
        feature_names: &[impl AsRef<str>],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<serde_json::Value, String> {
        let feature_names = self.get_filtered_feature_names(feature_names, preferences)?;
        let mut value = self.get_entire_config(&feature_names, cache_options, preferences)?;
        if preferences
            .map(|p| p.are_configurable_values_enabled())
            // Configurable strings are disabled by default.
            .unwrap_or(false)
        {
            // Strings need to be processed before lists because lists may contain strings.
            self.process_configurable_strings(&mut value, None)?;
            self.process_configurable_lists(&mut value, None)?;
        }
        Ok(value)
    }

    fn get_canonical_feature_name(&self, feature_name: &str) -> Result<String, String> {
        // Canonical feature names are also included as keys in the aliases map.
        let feature_name = unicase::UniCase::new(feature_name.to_owned());
        match self.aliases.get(&feature_name) {
            Some(canonical_name) => Ok(canonical_name.to_owned()),
            None => Err(format!(
                "Feature name {feature_name:?} is not a known feature."
            )),
        }
    }

    fn get_canonical_feature_names(
        &self,
        feature_names: &[impl AsRef<str>],
    ) -> Result<Vec<String>, String> {
        feature_names
            .iter()
            .map(|name| self.get_canonical_feature_name(name.as_ref()))
            .collect()
    }

    fn get_feature_metadata(&self, canonical_feature_name: &str) -> Option<OptionsMetadata> {
        self.features.get(canonical_feature_name).cloned()
    }

    fn get_features(&self) -> Vec<String> {
        self.sources.keys().cloned().collect()
    }

    fn get_features_referencing_file(&self, relative_path: &str) -> Option<Vec<String>> {
        self.referenced_file_to_feature_names
            .as_ref()
            .and_then(|map| map.get(relative_path).cloned())
    }

    fn get_features_with_metadata(&self) -> Features {
        self.features.clone()
    }

    fn get_filtered_feature_names(
        &self,
        feature_names: &[impl AsRef<str>],
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Vec<String>, String> {
        let mut skip_feature_name_conversion = false;
        let mut constraints = None;
        if let Some(preferences) = preferences {
            skip_feature_name_conversion = preferences.skip_feature_name_conversion;
            constraints = preferences.constraints.as_ref();
        }

        let mut result = Vec::new();
        for feature_name in feature_names {
            // Check for an alias.
            let canonical_feature_name: String = if skip_feature_name_conversion {
                feature_name.as_ref().to_owned()
            } else {
                self.get_canonical_feature_name(feature_name.as_ref())?
            };

            if let Some(constraints) = constraints {
                let conditions = self.conditions.get(&canonical_feature_name);
                if !conditions
                    .map(|conditions| conditions.evaluate(constraints))
                    .unwrap_or(true)
                {
                    continue;
                }
            }
            result.push(canonical_feature_name);
        }

        Ok(result)
    }

    fn get_options(
        &self,
        key: &str,
        feature_names: &[impl AsRef<str>],
    ) -> Result<serde_json::Value, String> {
        self.get_options_with_preferences(key, feature_names, None, None)
    }

    fn get_options_with_preferences(
        &self,
        key: &str,
        feature_names: &[impl AsRef<str>],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<serde_json::Value, String> {
        if cache_options.is_some() {
            match self.get_options_from_cache(key, feature_names, cache_options, preferences) {
                Ok(Some(options)) => return Ok(options),
                Ok(None) => (),
                Err(e) => return Err(e),
            }
        }

        let filtered_feature_names = self.get_filtered_feature_names(feature_names, preferences)?;
        let mut value =
            self.get_options_for_key(key, &filtered_feature_names, feature_names, preferences)?;

        if preferences
            .map(|p| p.are_configurable_values_enabled())
            // Configurable strings are disabled by default.
            .unwrap_or(false)
        {
            // Strings need to be processed before lists because lists may contain strings.
            self.process_configurable_strings(&mut value, Some(key))?;
            self.process_configurable_lists(&mut value, Some(key))?;
        }

        if cache_options.is_some() {
            let are_configurable_strings_enabled = preferences
                .map(|p| p.are_configurable_strings_enabled)
                .unwrap_or(false);
            let cache_key = (
                key.to_owned(),
                filtered_feature_names.clone(),
                are_configurable_strings_enabled,
            );
            self.options_cache
                .write()
                .expect("the options cache lock should be held")
                .insert(cache_key, value.clone());
        }
        Ok(value)
    }

    fn has_conditions(&self, canonical_feature_name: &str) -> bool {
        self.conditions.contains_key(canonical_feature_name)
    }

    fn map_feature_names(
        &self,
        feature_names: &[impl AsRef<str>],
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Vec<Option<String>>, String> {
        let mut skip_feature_name_conversion = false;
        let mut constraints = None;
        if let Some(preferences) = preferences {
            skip_feature_name_conversion = preferences.skip_feature_name_conversion;
            constraints = preferences.constraints.as_ref();
        }

        let mut result = Vec::new();
        for feature_name in feature_names {
            // Check for an alias.
            let canonical_feature_name: String = if skip_feature_name_conversion {
                feature_name.as_ref().to_owned()
            } else {
                self.get_canonical_feature_name(feature_name.as_ref())?
            };

            if let Some(constraints) = constraints {
                let conditions = self.conditions.get(&canonical_feature_name);
                if !conditions
                    .map(|conditions| conditions.evaluate(constraints))
                    .unwrap_or(true)
                {
                    result.push(None);
                    continue;
                }
            }
            result.push(Some(canonical_feature_name));
        }

        Ok(result)
    }
}
