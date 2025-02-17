use config::{self};
use std::collections::HashMap;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

// We won't truly use files at runtime, we're just using fake files that are backed by strings because that's easy to use with the `config` library.
pub(crate) type SourceValue = config::File<config::FileSourceString, config::FileFormat>;

pub(crate) type Aliases = HashMap<unicase::UniCase<String>, String>;
pub(crate) type Sources = HashMap<String, SourceValue>;

pub struct GetOptionsPreferences {
    pub skip_feature_name_conversion: bool,
}

/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
pub struct OptionsProvider {
    aliases: Aliases,
    sources: Sources,
}

impl OptionsProvider {
    pub(crate) fn new(aliases: &Aliases, sources: &Sources) -> Self {
        OptionsProvider {
            aliases: aliases.clone(),
            sources: sources.clone(),
        }
    }

    // Map an alias or canonical feature name (perhaps derived from a file name) to a canonical feature name.
    // Canonical feature names map to themselves.
    //
    // @param feature_name The name of an alias or a feature.
    // @return The canonical feature name.
    pub fn get_canonical_feature_name(&self, feature_name: &str) -> Result<&String, String> {
        // Canonical feature names are also included as keys in the aliases map.
        let feature_name = unicase::UniCase::new(feature_name.to_owned());
        match self.aliases.get(&feature_name) {
            Some(canonical_name) => Ok(canonical_name),
            None => Err(format!(
                "The given feature {:?} was not found.",
                feature_name
            )),
        }
    }

    pub fn get_options(
        &self,
        key: &str,
        feature_names: &Vec<String>,
    ) -> Result<serde_json::Value, String> {
        self.get_option_with_preferences(key, feature_names, &None)
    }

    // TODO Add another method with caching
    // with an option to disable because we will not want to use the cache when calling from other languages because they should use their own caching
    // in order to avoid possible overhead and conversion.
    pub fn get_option_with_preferences(
        &self,
        key: &str,
        feature_names: &Vec<String>,
        preferences: &Option<GetOptionsPreferences>,
    ) -> Result<serde_json::Value, String> {
        let mut config_builder = config::Config::builder();
        let mut skip_feature_name_conversion = false;
        if let Some(_preferences) = preferences {
            skip_feature_name_conversion = _preferences.skip_feature_name_conversion;
        }
        for feature_name in feature_names {
            // Check for an alias.
            // Canonical feature names are also included as keys in the aliases map.
            let mut canonical_feature_name = feature_name;
            if !skip_feature_name_conversion {
                canonical_feature_name = self.get_canonical_feature_name(feature_name)?;
            }

            let source = match self.sources.get(canonical_feature_name) {
                Some(src) => src,
                // Should not happen.
                // All canonical feature names are included as keys in the sources map.
                // It could happen in the future if we allow aliases to be added directly, but we should try to validate them when the provider is built.
                None => {
                    return Err(format!(
                        "Feature name {:?} was not found.",
                        canonical_feature_name
                    ))
                }
            };
            config_builder = config_builder.add_source(source.clone());
        }
        let config = config_builder.build();

        match config {
            Ok(cfg) => match cfg.get(key) {
                Ok(value) => Ok(value),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}
