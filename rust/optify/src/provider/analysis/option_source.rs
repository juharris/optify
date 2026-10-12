use serde::Serialize;

/// A source of values in the options.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OptionSource {
    /// The value came from a feature.
    Feature {
        /// The canonical feature names from the requested feature to the feature that defined the value.
        /// For example, `["grandparent", "parent2", "base2"]` when the requested feature `grandparent` imports `parent2` which imports `base2`.
        lineage: Vec<String>,
    },
    /// The value came from the overrides in the preferences.
    Overrides,
}

impl OptionSource {
    /// Returns the canonical name of the feature that defined the value or `None` for overrides.
    #[must_use]
    pub fn defining_feature(&self) -> Option<&str> {
        match self {
            OptionSource::Feature { lineage } => lineage.last().map(String::as_str),
            OptionSource::Overrides => None,
        }
    }
}
