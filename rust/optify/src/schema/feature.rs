// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/FeatureConfiguration.cs

use serde::Deserialize;

pub(crate) type ConfigurationOptions = config::Value;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct OptionsMetadata {
    // TODO Add more props.
    pub aliases: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct FeatureConfiguration {
    pub metadata: OptionsMetadata,
    pub options: ConfigurationOptions,
}
