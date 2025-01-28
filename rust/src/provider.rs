use config::{self};
use std::collections::HashMap;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

// We won't truly use files at runtime, we're just using fake files that are backed by strings because that's easy to use with the `config` library.
pub(crate) type SourceValue = config::File<config::FileSourceString, config::FileFormat>;

pub(crate) type Sources = HashMap<unicase::UniCase<String>, SourceValue>;

pub(crate) type Aliases = HashMap<unicase::UniCase<String>, unicase::UniCase<String>>;

/// ⚠️ Development in progress ⚠️\
/// Not truly considered public and mainly available to support bindings for other languages.
pub struct OptionsProvider {
    aliases: Aliases,
    sources: Sources,
}

impl OptionsProvider {
    pub fn new(aliases: Aliases, sources: Sources) -> Self {
        OptionsProvider { aliases, sources }
    }

    pub fn get_options(
        &self,
        key: &str,
        feature_names: &Vec<String>,
        // TODO Use a more specific error type.
    ) -> Result<serde_json::Value, String> {
        // TODO Add caching with option to disable because we will not want to use the cache when calling from other languages because they should use their own caching
        // in order to avoid possible overhead and conversion.
        let mut config_builder = config::Config::builder();
        for feature_name in feature_names {
            let feature_name = unicase::UniCase::new(feature_name.clone());

            // Check for an alias.
            let feature_name = match self.aliases.get(&feature_name) {
                Some(alias) => alias,
                None => &feature_name,
            };

            let source = match self.sources.get(&feature_name) {
                Some(src) => src,
                None => return Err(format!("Feature name {:?} was not found.", feature_name)),
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
