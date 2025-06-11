use serde_json::Value;

use crate::{
    provider::{CacheOptions, Features, GetOptionsPreferences},
    schema::metadata::OptionsMetadata,
};

/// Trait defining the core functionality for an options provider
pub trait OptionsRegistry {
    /// Gets all aliases.
    fn get_aliases(&self) -> Vec<String>;

    /// Gets all aliases.
    fn get_features_and_aliases(&self) -> Vec<String>;

    /// Gets all options for the specified feature names
    fn get_all_options(
        &self,
        feature_names: &[&str],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Value, String>;

    /// Gets the canonical feature name for a given feature name or alias
    fn get_canonical_feature_name(&self, feature_name: &str) -> Result<String, String>;

    // Map aliases or canonical feature names (perhaps derived from a file names) to the canonical feature names.
    // Canonical feature names map to themselves.
    //
    // @param feature_names The names of aliases or features.
    // @return The canonical feature names.
    fn get_canonical_feature_names(&self, feature_names: &[&str]) -> Result<Vec<String>, String>;

    fn get_feature_metadata(&self, canonical_feature_name: &str) -> Option<OptionsMetadata>;

    fn get_features(&self) -> Vec<String>;

    fn get_features_with_metadata(&self) -> Features;

    /// Gets options for a specific key and feature names
    fn get_options(&self, key: &str, feature_names: &[&str]) -> Result<Value, String>;

    /// Gets options with preferences for a specific key and feature names
    fn get_options_with_preferences(
        &self,
        key: &str,
        feature_names: &[&str],
        cache_options: Option<&CacheOptions>,
        preferences: Option<&GetOptionsPreferences>,
    ) -> Result<Value, String>;
}
