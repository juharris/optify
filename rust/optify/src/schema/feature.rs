// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/FeatureConfiguration.cs

use serde::Deserialize;

use super::metadata::OptionsMetadata;

pub(crate) type ConfigurationOptions = config::Value;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct FeatureConfiguration {
    // TODO Think of a good name for imports, maybe change it.
    pub imports: Option<Vec<String>>,
    pub metadata: Option<OptionsMetadata>,
    pub options: Option<ConfigurationOptions>,
}
