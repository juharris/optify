use config;
use std::collections::HashMap;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs
// and https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/OptionsProviderWithDefaults.cs

pub struct OptionsProvider {
    sources: HashMap<String, config::Value>,
}

impl OptionsProvider {
    pub(crate) fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn get_options(
        &self,
        key: &str,
        feature_names: Vec<String>,
        // TODO Use a more specific error type.
    ) -> Result<serde_json::Value, String> {
        let config_builder = config::Config::builder();
        for feature_name in feature_names {
            // FIXME Find a generic enough way to pass a source that is not a file.
            let source = self.sources.get(&feature_name).unwrap();
            config_builder.add_source(source);
        }
        let config = config_builder.build();

        match config {
            Ok(cfg) => Ok(cfg.get(key).unwrap()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_source(&self, feature_name: &str) -> config::Value {
        todo!()
    }
}
