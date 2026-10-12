use serde::Serialize;

use super::OptionSource;

/// Where a value in the options came from.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionOrigin {
    /// Lower-priority sources that also set a value at the path (or at an ancestor on the way to the path) and lost, highest priority first.
    /// This explains why values from other features were not used.
    pub overridden: Vec<OptionSource>,

    /// A JSON pointer equal to the requested path, a descendant of it, or an ancestor of it for `OptionAnalysis.blocked_by`.
    pub path: String,

    pub source: OptionSource,
}
