use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::RwLock,
};

use crate::{
    builder::{OptionsProviderBuilder, OptionsRegistryBuilder},
    provider::GetOptionsPreferences,
    schema::{conditions::ConditionExpression, metadata::OptionsMetadata},
};

use super::OptionsRegistry;

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
pub(crate) type OptionsCache = HashMap<(String, Vec<String>), serde_json::Value>;

pub struct CacheOptions {}

/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
pub struct OptionsProvider {
    aliases: Aliases,
    conditions: Conditions,
    configurable_value_pointers: ConfigurableValuePointers,
    features: Features,
    sources: Sources,

    // Caches - using RwLock for thread-safe interior mutability
    entire_config_cache: RwLock<EntireConfigCache>,
    options_cache: RwLock<OptionsCache>,
}

fn feature_names_to_vec<T: AsRef<str>>(feature_names: &[T]) -> Vec<String> {
    feature_names
        .iter()
        .map(|s| s.as_ref().to_owned())
        .collect()
}

/// Get a value at a JSON pointer location
fn get_value_at_pointer(value: &serde_json::Value, pointer: &str) -> Option<serde_json::Value> {
    value.pointer(pointer).cloned()
}

/// Set a value at a JSON pointer location
fn set_value_at_pointer(
    value: &mut serde_json::Value,
    pointer: &str,
    new_value: serde_json::Value,
) -> Result<(), String> {
    // Split pointer into parts
    let parts: Vec<&str> = pointer.split('/').skip(1).collect(); // Skip empty first part

    if parts.is_empty() {
        *value = new_value;
        return Ok(());
    }

    // FIXME There is probably a proper way to set the value without custom code.
    // Navigate to the parent and set the value
    let mut current = value;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            match current {
                serde_json::Value::Object(map) => {
                    map.insert(part.to_string(), new_value);
                    return Ok(());
                }
                serde_json::Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        if index < arr.len() {
                            arr[index] = new_value;
                            return Ok(());
                        }
                    }
                    return Err(format!("Invalid array index in pointer: {}", pointer));
                }
                _ => return Err(format!("Cannot set value at pointer: {}", pointer)),
            }
        } else {
            // Navigate deeper
            current = match current {
                serde_json::Value::Object(map) => map
                    .get_mut(*part)
                    .ok_or_else(|| format!("Path not found in pointer: {}", pointer))?,
                serde_json::Value::Array(arr) => {
                    let index = part
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid array index in pointer: {}", pointer))?;
                    arr.get_mut(index).ok_or_else(|| {
                        format!("Array index out of bounds in pointer: {}", pointer)
                    })?
                }
                _ => return Err(format!("Cannot navigate pointer: {}", pointer)),
            };
        }
    }

    Ok(())
}

impl OptionsProvider {
    pub(crate) fn new(
        aliases: &Aliases,
        conditions: &Conditions,
        configurable_value_pointers: &ConfigurableValuePointers,
        features: &Features,
        sources: &Sources,
    ) -> Self {
        OptionsProvider {
            aliases: aliases.clone(),
            conditions: conditions.clone(),
            configurable_value_pointers: configurable_value_pointers.clone(),
            features: features.clone(),
            sources: sources.clone(),
            entire_config_cache: RwLock::new(EntireConfigCache::new()),
            options_cache: RwLock::new(OptionsCache::new()),
        }
    }

    fn get_entire_config(
        &self,
        feature_names: &[impl AsRef<str>],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<config::Config, String> {
        let feature_names = self.get_filtered_feature_names(feature_names, preferences)?;
        if let Some(_cache_options) = cache_options {
            match self.get_entire_config_from_cache(&feature_names, preferences) {
                Ok(Some(config)) => return Ok(config),
                Ok(None) => (),
                Err(e) => return Err(e),
            }
        };
        let mut config_builder = config::Config::builder();
        for canonical_feature_name in feature_names.as_slice() {
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
                    let cache_key = feature_names;
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
        feature_names: &[impl AsRef<str>],
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Option<config::Config>, String> {
        if let Some(preferences) = preferences {
            if preferences.overrides_json.is_some() {
                return Err("Caching when overrides are given is not supported.".to_owned());
            }
        }
        let features = feature_names_to_vec(feature_names);
        let cache_key = features;
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
        let feature_names = self.get_filtered_feature_names(feature_names, preferences)?;
        let cache_key = (key.to_owned(), feature_names);
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

    /// Process configurable strings in the JSON value based on the pointers
    pub fn process_configurable_strings(
        &self,
        // FIXME Try not to copy the value. Can we pass by reference? Probably need to.
        mut value: serde_json::Value,
        feature_names: &[String],
    ) -> Result<serde_json::Value, String> {
        // TODO See if there is a more efficient way with less copying.
        // Also there should be a value to set a value at a pointer location.
        use crate::configurable_string::ConfigurableString;

        // Collect all configurable value pointers for the requested features
        let mut all_pointers = HashSet::new();
        for feature_name in feature_names {
            if let Some(pointers) = self.configurable_value_pointers.get(feature_name) {
                all_pointers.extend(pointers.iter().cloned());
            }
        }

        // If no configurable strings, return the value as-is
        if all_pointers.is_empty() {
            return Ok(value);
        }

        // For now, we'll use an empty LoadedFiles map
        // In the future, we might want to load files based on the configuration
        let loaded_files = HashMap::new();

        // Process each pointer
        for pointer in all_pointers {
            // Get the value at the pointer location
            let configurable_value = match get_value_at_pointer(&value, &pointer) {
                Some(v) => v,
                None => continue, // Skip if pointer doesn't exist
            };

            // Deserialize into ConfigurableString
            // FIXME Maybe we should deserialize and it would be faster to process the JSON directly?
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

            // Build the string
            let built_string = configurable_string.build(&loaded_files)?;

            // Replace the value at the pointer location with the built string
            set_value_at_pointer(
                &mut value,
                &pointer,
                serde_json::Value::String(built_string),
            )?;
        }

        Ok(value)
    }
}

impl OptionsRegistry for OptionsProvider {
    fn build(directory: impl AsRef<Path>) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(directory.as_ref())?;
        builder.build()
    }

    fn build_with_schema(
        directory: impl AsRef<Path>,
        schema_path: impl AsRef<Path>,
    ) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.with_schema(schema_path.as_ref())?;
        builder.add_directory(directory.as_ref())?;
        builder.build()
    }

    fn build_from_directories(directories: &[impl AsRef<Path>]) -> Result<OptionsProvider, String> {
        let mut builder = OptionsProviderBuilder::new();
        for directory in directories {
            builder.add_directory(directory.as_ref())?;
        }
        builder.build()
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
        builder.build()
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
        let config = self.get_entire_config(feature_names, cache_options, preferences)?;

        match config.try_deserialize() {
            Ok(value) => Ok(value),
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

        let config = self.get_entire_config(feature_names, cache_options, preferences)?;

        match config.get::<serde_json::Value>(key) {
            Ok(value) => {
                if let Some(_cache_options) = cache_options {
                    let feature_names =
                        self.get_filtered_feature_names(feature_names, preferences)?;
                    let cache_key = (key.to_owned(), feature_names);
                    self.options_cache
                        .write()
                        .expect("the options cache lock should be held")
                        .insert(cache_key, value.clone());
                }
                Ok(value)
            }
            Err(e) => {
                let feature_names = feature_names_to_vec(feature_names);
                Err(format!(
                    "Error getting options with features {feature_names:?}: {e}"
                ))
            }
        }
    }

    fn has_conditions(&self, canonical_feature_name: &str) -> bool {
        self.conditions.contains_key(canonical_feature_name)
    }
}
