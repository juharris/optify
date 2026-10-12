use serde_json::Value;

use super::explain::{explain, NotFound};
use super::replay::replay;
use super::{AnalysisData, OptionAnalysis};
use crate::json::parse_json_pointer::parse_json_pointer;

/// Explains which feature (and import chain) set the value at `key_path` for the canonical `feature_names`.
///
/// `feature_names` must already be filtered and converted to canonical feature names.
/// `original_feature_names` are only used for error messages.
pub(crate) fn analyze(
    data: &AnalysisData,
    key_path: &str,
    feature_names: &[String],
    original_feature_names: &[impl AsRef<str>],
    overrides: Option<&Value>,
) -> Result<OptionAnalysis, String> {
    let segments = parse_json_pointer(key_path)?;
    let replayed = replay(data, key_path, &segments, feature_names, overrides)?;
    explain(&replayed, &segments)
        .map_err(|not_found| format_not_found(&not_found, key_path, original_feature_names))
}

fn format_not_found(
    not_found: &NotFound,
    key_path: &str,
    original_feature_names: &[impl AsRef<str>],
) -> String {
    let feature_names: Vec<&str> = original_feature_names.iter().map(AsRef::as_ref).collect();
    let hint = not_found
        .non_object_path
        .as_ref()
        .map(|path| format!(" The value at \"{path}\" is not an object."))
        .unwrap_or_default();
    format!(
        "Error analyzing options with features {feature_names:?}: configuration property at \"{key_path}\" not found.{hint}"
    )
}
