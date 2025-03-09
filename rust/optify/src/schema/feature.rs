// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/FeatureConfiguration.cs

use serde::Deserialize;

use super::metadata::OptionsMetadata;

pub(crate) type ConfigurationOptions = config::Value;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct FeatureConfiguration {
    pub imports: Option<Vec<String>>,
    pub metadata: Option<OptionsMetadata>,
    pub options: Option<ConfigurationOptions>,
}
