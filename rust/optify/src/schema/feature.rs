use serde::Deserialize;

use super::conditions::ConditionExpression;
use super::metadata::OptionsMetadata;
use crate::policies::Policies;

pub(crate) type ConfigurationOptions = serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct FeatureConfiguration {
    pub imports: Option<Vec<String>>,
    pub metadata: Option<OptionsMetadata>,
    /// Conditions to automatically enable this feature file when constraints are given when getting configuration options.
    ///
    /// More details in the JSON schema.
    pub conditions: Option<ConditionExpression>,
    /// Policies that restrict which requesters may use this feature.
    ///
    /// More details in the JSON schema.
    pub policies: Option<Policies>,
    pub options: Option<ConfigurationOptions>,
}
