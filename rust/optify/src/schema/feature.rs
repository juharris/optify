// Similar to https://github.com/juharris/dotnet-OptionsProvider/blob/main/src/OptionsProvider/OptionsProvider/FeatureConfiguration.cs

use serde::Deserialize;

use super::conditions::ConditionExpression;
use super::metadata::OptionsMetadata;

pub(crate) type ConfigurationOptions = config::Value;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct FeatureConfiguration {
    pub imports: Option<Vec<String>>,
    pub metadata: Option<OptionsMetadata>,
    /// Conditions to automatically enable this feature file when constraints are given when getting configuration options.
    ///
    /// More details in the JSON schema.
    pub conditions: Option<ConditionExpression>,
    pub options: Option<ConfigurationOptions>,
}
