use config::{self};
use std::collections::HashMap;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

// We won't truly use files at runtime, we're just using fake files that are backed by strings because that's easy to use with the `config` library.
pub(crate) type SourceValue = config::File<config::FileSourceString, config::FileFormat>;

pub struct OptionsProvider {
    sources: HashMap<String, SourceValue>,
}

impl OptionsProvider {
    pub fn new(sources: std::collections::HashMap<String, SourceValue>) -> Self {
        OptionsProvider { sources }
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
            // TODO Look up an alias.
            let source = match self.sources.get(feature_name) {
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
