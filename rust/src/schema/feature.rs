// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/FeatureConfiguration.cs

use serde::Deserialize;

pub(crate) type ConfigurationOptions = config::Value;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct FeatureConfiguration {
    // TODO Add `metadata` with custom class.
    pub options: ConfigurationOptions,
}
