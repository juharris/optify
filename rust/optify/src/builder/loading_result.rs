use crate::schema::{conditions::ConditionExpression, metadata::OptionsMetadata};

/// The result of loading a feature configuration file.
pub(crate) struct LoadingResult {
    pub canonical_feature_name: String,
    pub conditions: Option<ConditionExpression>,
    pub configurable_value_pointers: Vec<String>,
    pub imports: Option<Vec<String>>,
    pub metadata: OptionsMetadata,
    pub original_config: serde_json::Value,
    pub source: config::File<config::FileSourceString, config::FileFormat>,
}
