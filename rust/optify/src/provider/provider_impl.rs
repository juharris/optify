use std::{collections::HashMap, path::Path};

use crate::{
    builder::{OptionsProviderBuilder, OptionsRegistryBuilder},
    provider::constraints::Constraints,
    schema::{conditions::ConditionExpression, metadata::OptionsMetadata},
};

use super::OptionsRegistry;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

// We won't truly use files at runtime, we're just using fake files that are backed by strings because that's easy to use with the `config` library.
pub(crate) type SourceValue = config::File<config::FileSourceString, config::FileFormat>;

pub(crate) type Aliases = HashMap<unicase::UniCase<String>, String>;
pub(crate) type Conditions = HashMap<String, ConditionExpression>;
pub(crate) type Features = HashMap<String, OptionsMetadata>;
pub(crate) type Sources = HashMap<String, SourceValue>;

pub struct GetOptionsPreferences {
    /// Overrides to apply after the built configuration.
    /// A string is used because it makes it easier to pass to the `config` library, but this may change in the future.
    /// It also makes it simpler and maybe faster to get from other programming languages.
    pub constraints: Option<Constraints>,
    pub overrides_json: Option<String>,
    pub skip_feature_name_conversion: bool,
}

impl Clone for GetOptionsPreferences {
    fn clone(&self) -> Self {
        Self {
            constraints: self.constraints.clone(),
            overrides_json: self.overrides_json.clone(),
            skip_feature_name_conversion: self.skip_feature_name_conversion,
        }
    }
}

impl Default for GetOptionsPreferences {
    fn default() -> Self {
        Self::new()
    }
}

impl GetOptionsPreferences {
    pub fn new() -> Self {
        Self {
            constraints: None,
            overrides_json: None,
            skip_feature_name_conversion: false,
        }
    }

    pub fn set_constraints(&mut self, constraints: Option<serde_json::Value>) {
        self.constraints = constraints.map(|c| Constraints { constraints: c });
    }

    pub fn set_constraints_json(&mut self, constraints: Option<&str>) {
        self.constraints = constraints.map(|c| Constraints {
            constraints: serde_json::from_str(c).expect("constraints should be valid JSON"),
        });
    }
}

pub struct CacheOptions {}

/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
pub struct OptionsProvider {
    aliases: Aliases,
    conditions: Conditions,
    features: Features,
    sources: Sources,
}

impl OptionsProvider {
    pub(crate) fn new(
        aliases: &Aliases,
        conditions: &Conditions,
        features: &Features,
        sources: &Sources,
    ) -> Self {
        OptionsProvider {
            aliases: aliases.clone(),
            conditions: conditions.clone(),
            features: features.clone(),
            sources: sources.clone(),
        }
    }

    fn get_entire_config(
        &self,
        feature_names: &[impl AsRef<str>],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<config::Config, String> {
        if let Some(_cache_options) = cache_options {
            if let Some(preferences) = preferences {
                if preferences.overrides_json.is_some() {
                    return Err("Caching is not supported yet and caching when overrides are given will not be supported.".to_owned());
                }
            }
            return Err("Caching is not supported yet.".to_owned());
        };
        let mut config_builder = config::Config::builder();
        let mut skip_feature_name_conversion = false;
        let mut constraints = None;
        if let Some(_preferences) = preferences {
            skip_feature_name_conversion = _preferences.skip_feature_name_conversion;
            constraints = _preferences.constraints.as_ref();
        }
        for feature_name in feature_names {
            // Check for an alias.
            // Canonical feature names are also included as keys in the aliases map.
            let canonical_feature_name = if skip_feature_name_conversion {
                feature_name.as_ref()
            } else {
                &self.get_canonical_feature_name(feature_name.as_ref())?
            };

            if let Some(constraints) = constraints {
                let conditions = self.conditions.get(canonical_feature_name);
                if !conditions
                    .map(|conditions| conditions.evaluate(constraints))
                    .unwrap_or(true)
                {
                    continue;
                }
            }

            let source = match self.sources.get(canonical_feature_name) {
                Some(src) => src,
                // Should not happen.
                // All canonical feature names are included as keys in the sources map.
                // It could happen in the future if we allow aliases to be added directly, but we should try to validate them when the provider is built.
                None => {
                    return Err(format!(
                        "Feature name {canonical_feature_name:?} is not a known feature."
                    ))
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
            Ok(cfg) => Ok(cfg),
            Err(e) => Err(format!(
                "Error combining features to build the configuration: {e}"
            )),
        }
    }
}

impl OptionsRegistry for OptionsProvider {
    fn build(directory: impl AsRef<Path>) -> Result<Self, String> {
        let mut builder = OptionsProviderBuilder::new();
        builder.add_directory(directory.as_ref())?;
        builder.build()
    }

    fn build_from_directories(directories: &[impl AsRef<Path>]) -> Result<Self, String> {
        let mut builder = OptionsProviderBuilder::new();
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
        let config = self.get_entire_config(feature_names, cache_options, preferences)?;

        match config.get(key) {
            Ok(value) => Ok(value),
            Err(e) => {
                let features = feature_names
                    .iter()
                    .map(|name| name.as_ref())
                    .collect::<Vec<_>>();
                Err(format!(
                    "Error getting options with features {features:?}: {e}"
                ))
            }
        }
    }

    fn get_possible_keys(
        &self,
        json_pointer: impl AsRef<str>,
        _cache_options: Option<&CacheOptions>,
    ) -> Vec<String> {
        let json_pointer = json_pointer.as_ref();
        let mut all_keys = std::collections::HashSet::new();

        for source in self.sources.values() {
            let config = match config::Config::builder().add_source(source.clone()).build() {
                Ok(cfg) => cfg,
                Err(_) => continue,
            };

            let value: serde_json::Value = match config.try_deserialize() {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(serde_json::Value::Object(map)) = value.pointer(json_pointer) {
                all_keys.extend(map.keys().cloned());
            }
        }

        let mut result: Vec<String> = all_keys.into_iter().collect();
        result.sort();

        // TODO: Cache

        result
    }

    fn get_possible_values(
        &self,
        _json_pointer: impl AsRef<str>,
        _cache_options: Option<&CacheOptions>,
    ) -> Vec<serde_json::Value> {
        todo!()
    }
}
