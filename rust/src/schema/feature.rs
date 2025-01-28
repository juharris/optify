// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/FeatureConfiguration.cs

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct FeatureConfiguration {
    // Ignore `metadata` for now.
    options: serde_json::Value,
}
