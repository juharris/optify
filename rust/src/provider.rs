use config::{self};
use std::collections::HashMap;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

pub(crate) type SourceValue = serde_json::Value;

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
        let mut config_builder = config::Config::builder();
        for feature_name in feature_names {
            let source = self.sources.get(feature_name).unwrap();
            // FIXME Get source passed in.
            config_builder = config_builder.add_source(source);
        }
        let config = config_builder.build();

        match config {
            Ok(cfg) => Ok(cfg.get(key).unwrap()),
            Err(e) => Err(e.to_string()),
        }
    }
}
