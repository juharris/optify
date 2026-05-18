use crate::{
    provider::SourceValue,
    schema::{conditions::ConditionExpression, metadata::OptionsMetadata},
};

/// The result of loading a feature configuration file.
pub(crate) struct LoadingResult {
    pub canonical_feature_name: String,
    pub conditions: Option<ConditionExpression>,
    /// A list of file paths that are explicitly referenced within this feature's ConfigurableStrings.
    /// This is only populated if the builder enables tracking.
    pub configurable_string_files: Vec<String>,
    pub configurable_value_pointers: Vec<String>,
    pub imports: Option<Vec<String>>,
    pub metadata: OptionsMetadata,
    pub original_config: serde_json::Value,
    pub source: SourceValue,
}
