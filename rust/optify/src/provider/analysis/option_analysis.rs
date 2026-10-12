use serde::Serialize;

use super::OptionOrigin;

/// Explains where the value at a path in the options came from.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionAnalysis {
    /// Set when features define a value at the path, but it is not in the final options because of a non-object value at an ancestor of the path.
    /// `path` is the ancestor, `source` set the non-object value, and `overridden` lists the sources whose values at the path were blocked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<OptionOrigin>,

    /// Where the value at the path came from.
    /// There is exactly one entry when one origin supplied the entire value at the path.
    /// This is always the case for non-object values (strings, numbers, booleans, null, and arrays) and its `path` is the requested path.
    /// When the value is an object merged from several origins, there is one entry per highest-level sub-path with a single origin, sorted by path.
    /// Empty when `blocked_by` is set.
    pub origins: Vec<OptionOrigin>,
}
