pub struct OptionsProvider;

// Replicating https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/IOptionsProvider.cs

impl OptionsProvider {
    pub fn get_options(&self, key: &str, feature_names: Vec<String>) -> serde_json::Value {
        // TODO: Implement this function
        serde_json::Value::Array(feature_names.into_iter().map(serde_json::Value::String).collect())
    }
}
