use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::RwLock,
};

use crate::{
    builder::{OptionsProviderBuilder, OptionsRegistryBuilder},
    configurable_string::LoadedFiles,
    provider::GetOptionsPreferences,
    schema::{conditions::ConditionExpression, metadata::OptionsMetadata},
};

use super::OptionsRegistry;
use crate::configurable_string::ConfigurableString;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

// We won't truly use files at runtime, we're just using fake files that are backed by strings because that's easy to use with the `config` library.
pub(crate) type SourceValue = config::File<config::FileSourceString, config::FileFormat>;

pub(crate) type Aliases = HashMap<unicase::UniCase<String>, String>;
pub(crate) type Conditions = HashMap<String, ConditionExpression>;
pub(crate) type ConfigurableValuePointers = HashMap<String, Vec<String>>;
pub(crate) type Features = HashMap<String, OptionsMetadata>;
pub(crate) type Sources = HashMap<String, SourceValue>;

pub(crate) type EntireConfigCache = HashMap<Vec<String>, config::Config>;
pub(crate) type OptionsCache = HashMap<(String, Vec<String>, bool), serde_json::Value>;

pub struct CacheOptions {}

/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
pub struct OptionsProvider {
    aliases: Aliases,
    conditions: Conditions,
    configurable_value_pointers: ConfigurableValuePointers,
    features: Features,
    loaded_files: LoadedFiles,
    sources: Sources,

    // Caches - using RwLock for thread-safe interior mutability
    entire_config_cache: RwLock<EntireConfigCache>,
    options_cache: RwLock<OptionsCache>,
}

impl OptionsProvider {
    pub(crate) fn new(
        aliases: Aliases,
        conditions: Conditions,
        configurable_value_pointers: ConfigurableValuePointers,
        features: Features,
        loaded_files: LoadedFiles,
        sources: Sources,
    ) -> Self {
        OptionsProvider {
            aliases,
            conditions,
            configurable_value_pointers,
            features,
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
    ) -> Result<config::Config, String> {
        if let Some(_cache_options) = cache_options {
            match self.get_entire_config_from_cache(feature_names, preferences) {
                Ok(Some(config)) => return Ok(config),
                Ok(None) => (),
                Err(e) => return Err(e),
            }
        };
        let mut config_builder = config::Config::builder();
        for canonical_feature_name in feature_names {
            let source = match self.sources.get(canonical_feature_name) {
                Some(src) => src,
                None => {
                    // Should not happen.
                    // All canonical feature names are included as keys in the sources map.
                    // It could happen in the future if we allow aliases to be added directly, but we should try to validate them when the provider is built.
                    return Err(format!(
                        "Feature name {canonical_feature_name:?} is not a known feature."
                    ));
                }
            };
            config_builder = config_builder.add_source(source.clone());
        }
        if let Some(preferences) = preferences {
            if let Some(overrides) = &preferences.overrides_json {
                config_builder = config_builder
                    .add_source(config::File::from_str(overrides, config::FileFormat::Json));
            }
        }

        match config_builder.build() {
            Ok(cfg) => {
                if let Some(_cache_options) = cache_options {
                    let cache_key = feature_names.to_owned();
                    self.entire_config_cache
                        .write()
                        .expect("the entire config cache lock should be held")
                        .insert(cache_key, cfg.clone());
                }
                Ok(cfg)
            }
            Err(e) => Err(format!(
                "Error combining features to build the configuration: {e}"
            )),
        }
    }

    fn get_entire_config_from_cache(
        &self,
        feature_names: &[String],
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Option<config::Config>, String> {
        if let Some(preferences) = preferences {
            if preferences.overrides_json.is_some() {
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

    /// Process configurable strings in the JSON value based on the pointers.
    pub fn process_configurable_strings(
        &self,
        value: &mut serde_json::Value,
        feature_names: &[String],
        key_prefix: Option<&str>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<(), String> {
        if preferences
            .map(|p| !p.are_configurable_strings_enabled)
            // Configurable strings are disabled by default.
            .unwrap_or(true)
        {
            return Ok(());
        }

        if self.configurable_value_pointers.is_empty() {
            // There are no configurable strings to process, assume they are not enabled for the provider, even for use in overrides.
            return Ok(());
        }

        // When overrides are present, dynamically discover all configurable strings
        // because overrides may contain ConfigurableStrings not in the original features
        let has_overrides = preferences
            .map(|p| p.overrides_json.is_some())
            .unwrap_or(false);

        // TODO It would be nice to save memory and use a `HashSet<&String>` instead of a `HashSet<String>`.
        // `HashSet<String>` is needed when there are overrides, but that should be the exception rather than the rule.
        let pointers: HashSet<String> = if has_overrides {
            // Dynamically find all configurable strings in the merged value
            crate::configurable_string::locator::find_configurable_values(Some(value))
                .into_iter()
                .collect()
        } else {
            // Collect all configurable pointers for the requested features.
            feature_names
                .iter()
                .filter_map(|feature_name| self.configurable_value_pointers.get(feature_name))
                .flat_map(|pointers| pointers.iter())
                .cloned()
                .collect()
        };

        if pointers.is_empty() {
            // There are no configurable strings to process.
            return Ok(());
        }

        for pointer in &pointers {
            let relative_pointer = match key_prefix {
                Some(key_prefix) => {
                    if !pointer.starts_with(key_prefix) {
                        // The pointer does not start with the key prefix so it will not be used.
                        continue;
                    } else {
                        // Remove the key prefix because we need pointers relative the current key.
                        pointer[key_prefix.len()..].to_string()
                    }
                }
                // There is not key prefix when the entire configuration is requested.
                _ => format!("/{}", pointer),
            };

            if let Some(configurable_value) = value.pointer_mut(&relative_pointer) {
                // Only continue if it has the right indicator property because it may have been overridden.
                if let Some(type_value) =
                    configurable_value.get(crate::configurable_string::locator::TYPE_KEY)
                {
                    if let Some(type_str) = type_value.as_str() {
                        if type_str != crate::configurable_string::locator::TYPE {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
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

                // Replace the value at the pointer location with the built string

                let built_string = configurable_string.build(&self.loaded_files)?;
                *configurable_value = serde_json::Value::String(built_string);
            }
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

    fn build_with_schema(
        directory: impl AsRef<Path>,
        schema_path: impl AsRef<Path>,
    ) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.with_schema(schema_path.as_ref())?;
        builder.add_directory(directory.as_ref())?;
        builder.build_and_clear()
    }

    fn build_from_directories(directories: &[impl AsRef<Path>]) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
        builder.build_and_clear()
    }

    fn build_from_directories_with_schema(
        directories: &[impl AsRef<Path>],
        schema_path: impl AsRef<Path>,
    ) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.with_schema(schema_path.as_ref())?;
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
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
        let config = self.get_entire_config(&feature_names, cache_options, preferences)?;

        match config.try_deserialize::<serde_json::Value>() {
            Ok(mut value) => {
                self.process_configurable_strings(&mut value, &feature_names, None, preferences)?;
                Ok(value)
            }
            Err(e) => Err(e.to_string()),
        }
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
        if let Some(_preferences) = preferences {
            skip_feature_name_conversion = _preferences.skip_feature_name_conversion;
            constraints = _preferences.constraints.as_ref();
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
        if let Some(_cache_options) = cache_options {
            match self.get_options_from_cache(key, feature_names, cache_options, preferences) {
                Ok(Some(options)) => return Ok(options),
                Ok(None) => (),
                Err(e) => return Err(e),
            }
        }

        let filtered_feature_names = self.get_filtered_feature_names(feature_names, preferences)?;
        let config = self.get_entire_config(&filtered_feature_names, cache_options, preferences)?;

        match config.get::<serde_json::Value>(key) {
            Ok(mut value) => {
                self.process_configurable_strings(
                    &mut value,
                    &filtered_feature_names,
                    Some(key),
                    preferences,
                )?;
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
            Err(e) => Err(format!(
                "Error getting options with features {:?}: {e}",
                feature_names.iter().map(|f| f.as_ref()).collect::<Vec<_>>()
            )),
        }
    }

    fn has_conditions(&self, canonical_feature_name: &str) -> bool {
        self.conditions.contains_key(canonical_feature_name)
    }
}
