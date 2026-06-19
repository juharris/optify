use crate::{
    configurable_values::locator::ConfigurableValuePointers,
    provider::SourceValue,
    schema::{conditions::ConditionExpression, metadata::OptionsMetadata},
};

pub(crate) enum LoadingResult {
    Feature(FeatureLoadingResult),
    Raw(RawLoadingResult),
}

/// The result of loading a feature configuration file.
pub(crate) struct FeatureLoadingResult {
    pub canonical_feature_name: String,
    pub conditions: Option<ConditionExpression>,
    /// A list of file paths that are explicitly referenced within this feature's ConfigurableStrings.
    /// This is only populated if the builder enables tracking.
    pub configurable_string_files: Vec<String>,
    pub configurable_value_pointers: ConfigurableValuePointers,
    pub imports: Option<Vec<String>>,
    pub metadata: OptionsMetadata,
    pub original_config: serde_json::Value,
    pub source: SourceValue,
}

/// The result of loading a feature configuration file.
pub(crate) struct RawLoadingResult {
    pub contents: String,
    pub relative_path: String,
}
